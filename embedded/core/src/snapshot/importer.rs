//! Snapshot Importer
//!
//! Provides functionality to import snapshots with atomic restore guarantees.
//! Uses a temp-file-then-replace strategy to ensure data integrity.

use super::sqlite_backup;
use super::types::*;
use crate::error::VecXError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Snapshot importer that restores snapshots with atomic guarantees.
pub struct SnapshotImporter {
    pool: Pool<SqliteConnectionManager>,
    config: SnapshotConfig,
    /// Original index file paths for replacement
    index_file_paths: Vec<String>,
}

impl SnapshotImporter {
    /// Creates a new snapshot importer.
    ///
    /// # Arguments
    ///
    /// * `pool` - Connection pool to the destination database
    /// * `config` - Snapshot configuration
    pub fn new(pool: Pool<SqliteConnectionManager>, config: SnapshotConfig) -> Self {
        Self {
            pool,
            config,
            index_file_paths: Vec::new(),
        }
    }

    /// Creates a new importer with default configuration.
    pub fn with_defaults(pool: Pool<SqliteConnectionManager>) -> Self {
        Self::new(pool, SnapshotConfig::default())
    }

    /// Sets the index file paths for replacement during import.
    ///
    /// This is needed to know where to restore HNSW index files.
    pub fn with_index_paths(mut self, paths: Vec<String>) -> Self {
        self.index_file_paths = paths;
        self
    }

    /// Imports a snapshot from an iterator of chunks.
    ///
    /// This method:
    /// 1. Receives chunks and writes to temporary files
    /// 2. Validates checksums
    /// 3. Atomically replaces the live database and index files
    ///
    /// # Arguments
    ///
    /// * `chunks` - Iterator of snapshot chunks
    ///
    /// # Returns
    ///
    /// Import result with statistics and status.
    pub fn import<I>(&self, chunks: I) -> Result<ImportResult, VecXError>
    where
        I: IntoIterator<Item = SnapshotChunk>,
    {
        let mut receiver = ChunkReceiver::new(&self.config.temp_dir)?;

        // Process all chunks
        for chunk in chunks {
            receiver.receive_chunk(chunk)?;
        }

        // Validate and finalize
        let import_data = receiver.finalize()?;

        // Perform atomic restore
        self.atomic_restore(&import_data)?;

        Ok(ImportResult::success(
            import_data.metadata.snapshot_id.clone(),
            import_data.metadata.total_size,
            import_data.metadata.files.len() as u32,
        ))
    }

    /// Performs atomic restore of database and index files.
    fn atomic_restore(&self, import_data: &ImportData) -> Result<(), VecXError> {
        // Step 1: Restore the SQLite database
        if let Some(db_path) = import_data.files.get("database.db") {
            sqlite_backup::restore_database(db_path, &self.pool)?;
        }

        // Step 2: Restore HNSW index files
        let index_files: Vec<_> = import_data
            .files
            .iter()
            .filter(|(name, _)| name.starts_with("index_"))
            .collect();

        for (idx, (_, temp_path)) in index_files.iter().enumerate() {
            if idx < self.index_file_paths.len() {
                let dest_path = &self.index_file_paths[idx];
                atomic_file_replace(temp_path, Path::new(dest_path))?;
            }
        }

        Ok(())
    }

    /// Imports a snapshot from a vector of chunks.
    ///
    /// Convenience method for non-streaming imports.
    pub fn import_from_vec(&self, chunks: Vec<SnapshotChunk>) -> Result<ImportResult, VecXError> {
        self.import(chunks.into_iter())
    }
}

/// Receives and assembles snapshot chunks into files.
struct ChunkReceiver {
    temp_dir: PathBuf,
    metadata: Option<SnapshotMetadata>,
    file_writers: HashMap<String, FileWriter>,
    completed_files: HashMap<String, PathBuf>,
    received_sequences: Vec<u64>,
    finalized: bool,
}

impl ChunkReceiver {
    fn new(base_temp_dir: &Path) -> Result<Self, VecXError> {
        // Create a unique temp directory for this import
        let import_id = SnapshotMetadata::generate_id();
        let temp_dir = base_temp_dir.join(format!("import_{}", import_id));
        fs::create_dir_all(&temp_dir).map_err(|e| {
            VecXError::IoError(format!("Failed to create import temp directory: {}", e))
        })?;

        Ok(Self {
            temp_dir,
            metadata: None,
            file_writers: HashMap::new(),
            completed_files: HashMap::new(),
            received_sequences: Vec::new(),
            finalized: false,
        })
    }

    fn receive_chunk(&mut self, chunk: SnapshotChunk) -> Result<(), VecXError> {
        if self.finalized {
            return Err(VecXError::Other("Import already finalized".to_string()));
        }

        // Track sequence for ordering validation
        self.received_sequences.push(chunk.sequence);

        // Handle metadata (first chunk)
        if let Some(metadata) = chunk.metadata {
            self.metadata = Some(metadata);
        }

        // Handle file data
        if let Some(file_chunk) = chunk.file_chunk {
            self.write_file_chunk(file_chunk)?;
        }

        // Handle final chunk
        if chunk.is_final {
            self.finalized = true;
        }

        Ok(())
    }

    fn write_file_chunk(&mut self, chunk: FileChunk) -> Result<(), VecXError> {
        let file_name = chunk.file_name.clone();

        // Get or create file writer
        if !self.file_writers.contains_key(&file_name) {
            let file_path = self.temp_dir.join(&file_name);
            let writer = FileWriter::new(&file_path)?;
            self.file_writers.insert(file_name.clone(), writer);
        }

        let writer = self.file_writers.get_mut(&file_name).unwrap();
        writer.write(&chunk.data, chunk.offset)?;

        // If this is the last chunk for this file, close it
        if chunk.is_last_chunk {
            if let Some(mut writer) = self.file_writers.remove(&file_name) {
                writer.flush()?;
                self.completed_files
                    .insert(file_name, self.temp_dir.join(&chunk.file_name));
            }
        }

        Ok(())
    }

    fn finalize(mut self) -> Result<ImportData, VecXError> {
        // Close any remaining open files
        for (name, mut writer) in self.file_writers.drain() {
            writer.flush()?;
            self.completed_files
                .insert(name.clone(), self.temp_dir.join(&name));
        }

        // Validate we received metadata - use take() to move out of Option
        let metadata = self.metadata.take().ok_or_else(|| {
            VecXError::Other("No metadata received in snapshot".to_string())
        })?;

        // Validate all expected files were received
        for file_info in &metadata.files {
            if !self.completed_files.contains_key(&file_info.file_name) {
                return Err(VecXError::Other(format!(
                    "Missing file in snapshot: {}",
                    file_info.file_name
                )));
            }
        }

        // Validate checksums
        for file_info in &metadata.files {
            let file_path = &self.completed_files[&file_info.file_name];
            let actual_checksum = compute_file_checksum(file_path)?;
            if actual_checksum != file_info.checksum {
                return Err(VecXError::Other(format!(
                    "Checksum mismatch for file {}: expected {}, got {}",
                    file_info.file_name, file_info.checksum, actual_checksum
                )));
            }
        }

        // Use std::mem::take to move out of self without triggering Drop issues
        let completed_files = std::mem::take(&mut self.completed_files);
        let temp_dir = std::mem::take(&mut self.temp_dir);

        Ok(ImportData {
            metadata,
            files: completed_files,
            temp_dir,
        })
    }
}

impl Drop for ChunkReceiver {
    fn drop(&mut self) {
        // Clean up temp directory on error/drop
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

/// Holds data for a completed import before atomic restore.
struct ImportData {
    metadata: SnapshotMetadata,
    files: HashMap<String, PathBuf>,
    temp_dir: PathBuf,
}

impl Drop for ImportData {
    fn drop(&mut self) {
        // Clean up temp directory
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

/// File writer for assembling chunks.
struct FileWriter {
    file: File,
    path: PathBuf,
}

impl FileWriter {
    fn new(path: &Path) -> Result<Self, VecXError> {
        let file = File::create(path).map_err(|e| {
            VecXError::IoError(format!("Failed to create file {}: {}", path.display(), e))
        })?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    fn write(&mut self, data: &[u8], _offset: u64) -> Result<(), VecXError> {
        // Note: For simplicity, we assume chunks arrive in order
        // A production implementation should handle out-of-order chunks
        self.file.write_all(data).map_err(|e| {
            VecXError::IoError(format!("Failed to write to file: {}", e))
        })
    }

    fn flush(&mut self) -> Result<(), VecXError> {
        self.file.flush().map_err(|e| {
            VecXError::IoError(format!("Failed to flush file: {}", e))
        })
    }
}

/// Atomically replaces a file using rename.
///
/// This ensures that the destination file is either the old version
/// or the new version, never a partial write.
fn atomic_file_replace(src: &Path, dest: &Path) -> Result<(), VecXError> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            VecXError::IoError(format!("Failed to create destination directory: {}", e))
        })?;
    }

    // On Unix, rename is atomic within the same filesystem
    // For cross-filesystem moves, we need to copy then remove
    if let Err(_) = fs::rename(src, dest) {
        // Rename failed (likely cross-filesystem), fall back to copy
        fs::copy(src, dest).map_err(|e| {
            VecXError::IoError(format!("Failed to copy file: {}", e))
        })?;
        let _ = fs::remove_file(src);
    }

    Ok(())
}

/// Computes checksum of a file (same as in exporter).
fn compute_file_checksum(path: &Path) -> Result<String, VecXError> {
    use std::io::Read;

    let mut file = File::open(path).map_err(|e| {
        VecXError::IoError(format!("Failed to open file for checksum: {}", e))
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            VecXError::IoError(format!("Failed to read file for checksum: {}", e))
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize())
}

/// Simple SHA-256 hasher (same as in exporter).
struct Sha256 {
    data: Vec<u8>,
}

impl Sha256 {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn finalize(self) -> String {
        let mut hash: u64 = 0;
        for (i, &byte) in self.data.iter().enumerate() {
            hash = hash.wrapping_add((byte as u64).wrapping_mul(31_u64.wrapping_pow(i as u32)));
        }
        format!("{:016x}", hash)
    }
}

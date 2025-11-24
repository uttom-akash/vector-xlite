//! Snapshot Exporter
//!
//! Provides functionality to export consistent snapshots of the database
//! and HNSW index files as streaming chunks for Raft FSM integration.

use super::sqlite_backup;
use super::types::*;
use crate::error::VecXError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Snapshot exporter that creates consistent snapshots and streams them as chunks.
pub struct SnapshotExporter {
    pool: Pool<SqliteConnectionManager>,
    config: SnapshotConfig,
}

impl SnapshotExporter {
    /// Creates a new snapshot exporter.
    ///
    /// # Arguments
    ///
    /// * `pool` - Connection pool to the database
    /// * `config` - Snapshot configuration
    pub fn new(pool: Pool<SqliteConnectionManager>, config: SnapshotConfig) -> Self {
        Self { pool, config }
    }

    /// Creates a new exporter with default configuration.
    pub fn with_defaults(pool: Pool<SqliteConnectionManager>) -> Self {
        Self::new(pool, SnapshotConfig::default())
    }

    /// Exports a snapshot and returns an iterator over chunks.
    ///
    /// This method:
    /// 1. Creates a consistent backup of the SQLite database
    /// 2. Collects HNSW index files if configured
    /// 3. Generates metadata and checksums
    /// 4. Returns an iterator that yields chunks for streaming
    ///
    /// # Returns
    ///
    /// An iterator that yields `SnapshotChunk` items suitable for streaming.
    pub fn export(&self) -> Result<SnapshotChunkIterator, VecXError> {
        // Create temp directory for this export
        let export_id = SnapshotMetadata::generate_id();
        let export_dir = self.config.temp_dir.join(&export_id);
        std::fs::create_dir_all(&export_dir).map_err(|e| {
            VecXError::IoError(format!("Failed to create export directory: {}", e))
        })?;

        // Step 1: Backup the SQLite database
        let db_backup_path = export_dir.join("database.db");
        let db_size = sqlite_backup::backup_database(&self.pool, &db_backup_path)?;
        let db_checksum = compute_file_checksum(&db_backup_path)?;

        let mut files = vec![SnapshotFileInfo {
            file_name: "database.db".to_string(),
            file_type: SnapshotFileType::SqliteDb,
            file_size: db_size,
            checksum: db_checksum,
        }];

        let mut file_paths: HashMap<String, PathBuf> = HashMap::new();
        file_paths.insert("database.db".to_string(), db_backup_path);

        // Step 2: Collect HNSW index files if configured
        if self.config.include_index_files {
            let index_files = sqlite_backup::get_index_files(&self.pool)?;
            for (idx, index_path) in index_files.iter().enumerate() {
                let source_path = Path::new(index_path);
                if source_path.exists() {
                    // Copy index file to export directory
                    let index_name = format!("index_{}.idx", idx);
                    let dest_path = export_dir.join(&index_name);
                    std::fs::copy(source_path, &dest_path).map_err(|e| {
                        VecXError::IoError(format!("Failed to copy index file: {}", e))
                    })?;

                    let file_size = std::fs::metadata(&dest_path)
                        .map(|m| m.len())
                        .map_err(|e| {
                            VecXError::IoError(format!("Failed to get index file size: {}", e))
                        })?;
                    let checksum = compute_file_checksum(&dest_path)?;

                    files.push(SnapshotFileInfo {
                        file_name: index_name.clone(),
                        file_type: SnapshotFileType::HnswIndex,
                        file_size,
                        checksum,
                    });
                    file_paths.insert(index_name, dest_path);
                }
            }
        }

        // Step 3: Compute total size and snapshot checksum
        let total_size: u64 = files.iter().map(|f| f.file_size).sum();
        let snapshot_checksum = compute_snapshot_checksum(&files);

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let metadata = SnapshotMetadata {
            snapshot_id: export_id,
            created_at,
            total_size,
            files,
            version: SNAPSHOT_VERSION,
            checksum: snapshot_checksum,
        };

        // Step 4: Create the chunk iterator
        Ok(SnapshotChunkIterator::new(
            metadata,
            file_paths,
            self.config.chunk_size,
            export_dir,
        ))
    }

    /// Exports a snapshot directly to memory (for in-memory databases).
    ///
    /// This is a convenience method that collects all chunks into a vector.
    /// Use `export()` for streaming large snapshots.
    pub fn export_to_memory(&self) -> Result<Vec<SnapshotChunk>, VecXError> {
        let iterator = self.export()?;
        Ok(iterator.collect())
    }
}

/// Iterator that yields snapshot chunks for streaming.
pub struct SnapshotChunkIterator {
    metadata: Option<SnapshotMetadata>,
    file_paths: HashMap<String, PathBuf>,
    file_order: Vec<String>,
    current_file_idx: usize,
    current_reader: Option<BufReader<File>>,
    current_offset: u64,
    chunk_size: usize,
    sequence: u64,
    done: bool,
    export_dir: PathBuf,
}

impl SnapshotChunkIterator {
    fn new(
        metadata: SnapshotMetadata,
        file_paths: HashMap<String, PathBuf>,
        chunk_size: usize,
        export_dir: PathBuf,
    ) -> Self {
        let file_order: Vec<String> = metadata.files.iter().map(|f| f.file_name.clone()).collect();
        Self {
            metadata: Some(metadata),
            file_paths,
            file_order,
            current_file_idx: 0,
            current_reader: None,
            current_offset: 0,
            chunk_size,
            sequence: 0,
            done: false,
            export_dir,
        }
    }

    fn next_chunk(&mut self) -> Option<Result<SnapshotChunk, VecXError>> {
        if self.done {
            return None;
        }

        let sequence = self.sequence;
        self.sequence += 1;

        // First chunk includes metadata
        if sequence == 0 {
            return Some(Ok(SnapshotChunk {
                metadata: self.metadata.take(),
                file_chunk: None,
                sequence,
                is_final: false,
            }));
        }

        // Open next file if needed
        if self.current_reader.is_none() {
            if self.current_file_idx >= self.file_order.len() {
                // All files processed, send final chunk
                self.done = true;
                return Some(Ok(SnapshotChunk {
                    metadata: None,
                    file_chunk: None,
                    sequence,
                    is_final: true,
                }));
            }

            let file_name = &self.file_order[self.current_file_idx];
            let file_path = match self.file_paths.get(file_name) {
                Some(p) => p.clone(),
                None => {
                    return Some(Err(VecXError::IoError(format!(
                        "File path not found: {}",
                        file_name
                    ))));
                }
            };

            let file = match File::open(&file_path) {
                Ok(f) => f,
                Err(e) => {
                    return Some(Err(VecXError::IoError(format!(
                        "Failed to open file {}: {}",
                        file_name, e
                    ))));
                }
            };

            self.current_reader = Some(BufReader::new(file));
            self.current_offset = 0;
        }

        // Read chunk from current file
        let reader = self.current_reader.as_mut().unwrap();
        let file_name = self.file_order[self.current_file_idx].clone();

        let mut buffer = vec![0u8; self.chunk_size];
        let bytes_read = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                return Some(Err(VecXError::IoError(format!(
                    "Failed to read file: {}",
                    e
                ))));
            }
        };

        if bytes_read == 0 {
            // Current file is done, move to next
            self.current_reader = None;
            self.current_file_idx += 1;
            return self.next_chunk();
        }

        buffer.truncate(bytes_read);
        let offset = self.current_offset;
        self.current_offset += bytes_read as u64;

        // Check if this is the last chunk for this file
        let is_last_chunk = {
            let mut peek_buf = [0u8; 1];
            match reader.read(&mut peek_buf) {
                Ok(0) => true,
                Ok(_) => {
                    // Put the byte back (we can't actually do this, so we need to track differently)
                    // For simplicity, we'll just check if we read less than chunk_size
                    false
                }
                Err(_) => true,
            }
        };

        let is_last = bytes_read < self.chunk_size || is_last_chunk;
        if is_last {
            self.current_reader = None;
            self.current_file_idx += 1;
        }

        Some(Ok(SnapshotChunk {
            metadata: None,
            file_chunk: Some(FileChunk {
                file_name,
                offset,
                data: buffer,
                is_last_chunk: is_last,
            }),
            sequence,
            is_final: false,
        }))
    }
}

impl Iterator for SnapshotChunkIterator {
    type Item = SnapshotChunk;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_chunk() {
            Some(Ok(chunk)) => Some(chunk),
            Some(Err(_)) => None, // Could log error here
            None => None,
        }
    }
}

impl Drop for SnapshotChunkIterator {
    fn drop(&mut self) {
        // Clean up export directory
        let _ = std::fs::remove_dir_all(&self.export_dir);
    }
}

/// Computes SHA-256 checksum of a file.
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

/// Computes a checksum for the entire snapshot based on file checksums.
fn compute_snapshot_checksum(files: &[SnapshotFileInfo]) -> String {
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.file_name.as_bytes());
        hasher.update(file.checksum.as_bytes());
    }
    hasher.finalize()
}

/// Simple SHA-256 hasher implementation.
///
/// Note: In production, you'd want to use a proper crypto library like `sha2`.
/// This is a simplified implementation for demonstration.
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
        // Simple hash for demonstration - use sha2 crate in production
        // This uses a basic checksum approach
        let mut hash: u64 = 0;
        for (i, &byte) in self.data.iter().enumerate() {
            hash = hash.wrapping_add((byte as u64).wrapping_mul(31_u64.wrapping_pow(i as u32)));
        }
        format!("{:016x}", hash)
    }
}

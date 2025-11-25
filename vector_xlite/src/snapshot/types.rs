//! Snapshot types and configuration

use std::path::PathBuf;

/// Default chunk size for streaming snapshots (64 KB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Current snapshot format version
pub const SNAPSHOT_VERSION: u32 = 1;

/// Configuration for snapshot operations
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Chunk size for streaming in bytes
    pub chunk_size: usize,
    /// Whether to include HNSW index files
    pub include_index_files: bool,
    /// Temporary directory for atomic restore operations
    pub temp_dir: PathBuf,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            include_index_files: true,
            temp_dir: std::env::temp_dir(),
        }
    }
}

impl SnapshotConfig {
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_index_files(mut self, include: bool) -> Self {
        self.include_index_files = include;
        self
    }

    pub fn with_temp_dir(mut self, dir: PathBuf) -> Self {
        self.temp_dir = dir;
        self
    }
}

/// Type of file in a snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotFileType {
    /// SQLite database file
    SqliteDb,
    /// HNSW index file
    HnswIndex,
    /// WAL file (for consistency)
    Wal,
}

impl SnapshotFileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotFileType::SqliteDb => "sqlite_db",
            SnapshotFileType::HnswIndex => "hnsw_index",
            SnapshotFileType::Wal => "wal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "sqlite_db" => Some(SnapshotFileType::SqliteDb),
            "hnsw_index" => Some(SnapshotFileType::HnswIndex),
            "wal" => Some(SnapshotFileType::Wal),
            _ => None,
        }
    }
}

/// Information about a file in the snapshot
#[derive(Debug, Clone)]
pub struct SnapshotFileInfo {
    /// Name/path of the file
    pub file_name: String,
    /// Type of file
    pub file_type: SnapshotFileType,
    /// Size of the file in bytes
    pub file_size: u64,
    /// SHA-256 checksum of the file
    pub checksum: String,
}

/// Metadata about a complete snapshot
#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    /// Unique identifier for the snapshot
    pub snapshot_id: String,
    /// Unix timestamp in milliseconds when snapshot was created
    pub created_at: i64,
    /// Total size of all files in bytes
    pub total_size: u64,
    /// List of files in the snapshot
    pub files: Vec<SnapshotFileInfo>,
    /// Snapshot format version
    pub version: u32,
    /// SHA-256 checksum of the entire snapshot
    pub checksum: String,
}

impl SnapshotMetadata {
    /// Generate a new unique snapshot ID
    pub fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let random_part: u32 = {
            // Simple random using system time nanos
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);
            nanos ^ (nanos >> 16)
        };

        format!("snap_{}_{:08x}", timestamp, random_part)
    }
}

/// A chunk of snapshot data for streaming
#[derive(Debug, Clone)]
pub struct SnapshotChunk {
    /// Metadata (only present in first chunk)
    pub metadata: Option<SnapshotMetadata>,
    /// File chunk data (if any)
    pub file_chunk: Option<FileChunk>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// A chunk of file data
#[derive(Debug, Clone)]
pub struct FileChunk {
    /// Name of the file
    pub file_name: String,
    /// Offset within the file
    pub offset: u64,
    /// The actual data
    pub data: Vec<u8>,
    /// Whether this is the last chunk for this file
    pub is_last_chunk: bool,
}

/// Result of a snapshot import operation
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Whether import was successful
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// ID of the imported snapshot
    pub snapshot_id: String,
    /// Total bytes restored
    pub bytes_restored: u64,
    /// Number of files restored
    pub files_restored: u32,
}

impl ImportResult {
    pub fn success(snapshot_id: String, bytes_restored: u64, files_restored: u32) -> Self {
        Self {
            success: true,
            error_message: None,
            snapshot_id,
            bytes_restored,
            files_restored,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            error_message: Some(error),
            snapshot_id: String::new(),
            bytes_restored: 0,
            files_restored: 0,
        }
    }
}

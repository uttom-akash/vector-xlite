//! Snapshot module for VectorXLite
//!
//! Provides functionality for creating and restoring consistent snapshots
//! of the database and HNSW index files for Raft FSM integration.
//!
//! # Features
//!
//! - SQLite backup API integration for consistent database snapshots
//! - HNSW index file handling
//! - Streaming chunk support for large snapshots
//! - Atomic restore with temp file strategy
//!
//! # Usage
//!
//! ```rust,ignore
//! use vector_xlite::snapshot::{SnapshotManager, SnapshotConfig};
//!
//! let config = SnapshotConfig::default();
//! let manager = SnapshotManager::new(pool, config);
//!
//! // Export snapshot as chunks
//! let chunks = manager.export_snapshot()?;
//!
//! // Import snapshot from chunks
//! manager.import_snapshot(chunks)?;
//! ```

mod types;
mod exporter;
mod importer;
mod sqlite_backup;

pub use types::*;
pub use exporter::SnapshotExporter;
pub use importer::SnapshotImporter;

//! SQLite Backup API integration for consistent snapshots
//!
//! Uses SQLite's online backup API to create consistent snapshots
//! of both in-memory and on-disk databases.

use crate::error::VecXError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

/// Number of pages to copy per backup step
const PAGES_PER_STEP: i32 = 100;

/// Delay between backup steps in milliseconds
const STEP_DELAY_MS: u64 = 10;

/// Performs a backup of the database using SQLite's backup API.
///
/// This function creates a consistent point-in-time snapshot of the database,
/// including all tables, indexes, and data. It works with both in-memory
/// and file-based databases.
///
/// # Arguments
///
/// * `pool` - Connection pool to the source database
/// * `dest_path` - Path where the backup will be written
///
/// # Returns
///
/// The size of the backup file in bytes, or an error if backup failed.
pub fn backup_database(
    pool: &Pool<SqliteConnectionManager>,
    dest_path: &Path,
) -> Result<u64, VecXError> {
    // Get a connection from the pool
    let source_conn = pool.get().map_err(|e| {
        VecXError::Other(format!("Failed to get connection for backup: {}", e))
    })?;

    // Open destination database (mutable for backup API)
    let mut dest_conn = Connection::open(dest_path).map_err(|e| {
        VecXError::SqlError(format!("Failed to open destination database: {}", e))
    })?;

    // Perform the backup
    backup_connection(&source_conn, &mut dest_conn)?;

    // Get file size
    let file_size = std::fs::metadata(dest_path)
        .map(|m| m.len())
        .map_err(|e| VecXError::IoError(format!("Failed to get backup file size: {}", e)))?;

    Ok(file_size)
}

/// Performs backup from one connection to another using SQLite backup API.
fn backup_connection(
    source: &rusqlite::Connection,
    dest: &mut rusqlite::Connection,
) -> Result<(), VecXError> {
    // Use rusqlite's backup API
    let backup = rusqlite::backup::Backup::new(source, dest).map_err(|e| {
        VecXError::SqlError(format!("Failed to initialize backup: {}", e))
    })?;

    // Perform backup in steps to allow for progress tracking and
    // to avoid blocking the source database for too long
    loop {
        let step_result = backup.step(PAGES_PER_STEP).map_err(|e| {
            VecXError::SqlError(format!("Backup step failed: {}", e))
        })?;

        match step_result {
            rusqlite::backup::StepResult::Done => break,
            rusqlite::backup::StepResult::More => {
                // Small delay between steps to reduce contention
                std::thread::sleep(Duration::from_millis(STEP_DELAY_MS));
            }
            rusqlite::backup::StepResult::Busy => {
                // Wait a bit longer if busy
                std::thread::sleep(Duration::from_millis(STEP_DELAY_MS * 10));
            }
            rusqlite::backup::StepResult::Locked => {
                // Wait a bit longer if locked
                std::thread::sleep(Duration::from_millis(STEP_DELAY_MS * 10));
            }
            _ => {
                // Handle any future variants
                std::thread::sleep(Duration::from_millis(STEP_DELAY_MS));
            }
        }
    }

    Ok(())
}

/// Backs up an in-memory database to a byte vector.
///
/// This is useful for streaming snapshots where the database
/// content needs to be sent over the network.
///
/// # Arguments
///
/// * `pool` - Connection pool to the source in-memory database
///
/// # Returns
///
/// A byte vector containing the complete database backup.
pub fn backup_to_memory(pool: &Pool<SqliteConnectionManager>) -> Result<Vec<u8>, VecXError> {
    // Create a temporary file for the backup
    let temp_path = std::env::temp_dir().join(format!(
        "vxlite_backup_{}.db",
        std::process::id()
    ));

    // Perform backup to temp file
    let _size = backup_database(pool, &temp_path)?;

    // Read the file into memory
    let data = std::fs::read(&temp_path).map_err(|e| {
        VecXError::IoError(format!("Failed to read backup file: {}", e))
    })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(data)
}

/// Restores a database from a backup file.
///
/// # Arguments
///
/// * `backup_path` - Path to the backup file
/// * `dest_pool` - Connection pool to the destination database
///
/// # Note
///
/// This operation replaces the entire content of the destination database.
/// For file-based databases, consider using atomic file replacement instead.
pub fn restore_database(
    backup_path: &Path,
    dest_pool: &Pool<SqliteConnectionManager>,
) -> Result<(), VecXError> {
    // Open the backup file
    let backup_conn = Connection::open(backup_path).map_err(|e| {
        VecXError::SqlError(format!("Failed to open backup file: {}", e))
    })?;

    // Get a mutable connection to the destination
    let mut dest_conn = dest_pool.get().map_err(|e| {
        VecXError::Other(format!("Failed to get destination connection: {}", e))
    })?;

    // Perform restore (backup from file to destination)
    // We need to dereference to get the underlying Connection
    backup_connection(&backup_conn, &mut *dest_conn)?;

    Ok(())
}

/// Restores a database from a byte vector (for in-memory databases).
///
/// # Arguments
///
/// * `data` - The backup data as bytes
/// * `dest_pool` - Connection pool to the destination database
pub fn restore_from_memory(
    data: &[u8],
    dest_pool: &Pool<SqliteConnectionManager>,
) -> Result<(), VecXError> {
    // Write data to temp file first
    let temp_path = std::env::temp_dir().join(format!(
        "vxlite_restore_{}.db",
        std::process::id()
    ));

    std::fs::write(&temp_path, data).map_err(|e| {
        VecXError::IoError(format!("Failed to write temp restore file: {}", e))
    })?;

    // Restore from temp file
    let result = restore_database(&temp_path, dest_pool);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    result
}

/// Gets the list of HNSW index files associated with a database.
///
/// This scans the database for vectorlite virtual tables and extracts
/// their index file paths.
///
/// # Arguments
///
/// * `pool` - Connection pool to the database
///
/// # Returns
///
/// A vector of index file paths.
pub fn get_index_files(pool: &Pool<SqliteConnectionManager>) -> Result<Vec<String>, VecXError> {
    let conn = pool.get().map_err(|e| {
        VecXError::Other(format!("Failed to get connection: {}", e))
    })?;

    // Query sqlite_master for vectorlite virtual tables
    let mut stmt = conn
        .prepare(
            "SELECT sql FROM sqlite_master WHERE type='table' AND sql LIKE '%vectorlite%'",
        )
        .map_err(|e| VecXError::SqlError(format!("Failed to prepare query: {}", e)))?;

    let sql_strings: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| VecXError::SqlError(format!("Failed to query tables: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    // Extract index file paths from the SQL statements
    let mut index_files = Vec::new();
    for sql in sql_strings {
        // Parse the vectorlite CREATE VIRTUAL TABLE statement to find index file path
        // Format: CREATE VIRTUAL TABLE ... USING vectorlite(..., path/to/index.idx)
        if let Some(path) = extract_index_path(&sql) {
            if !path.is_empty() && path != ":memory:" {
                index_files.push(path);
            }
        }
    }

    Ok(index_files)
}

/// Extracts the index file path from a vectorlite CREATE VIRTUAL TABLE statement.
fn extract_index_path(sql: &str) -> Option<String> {
    // Look for the last argument in USING vectorlite(...)
    // The index file path is typically the last argument
    let sql_lower = sql.to_lowercase();
    let using_pos = sql_lower.find("using vectorlite(")?;
    let start = using_pos + "using vectorlite(".len();
    let end = sql[start..].find(')')? + start;
    let args = &sql[start..end];

    // Split by comma and take the last non-empty argument that looks like a path
    let parts: Vec<&str> = args.split(',').collect();
    for part in parts.iter().rev() {
        let trimmed = part.trim().trim_matches(|c| c == '\'' || c == '"');
        // Check if it looks like a file path (contains / or \ or ends with .idx)
        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.ends_with(".idx") {
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_index_path() {
        let sql = "CREATE VIRTUAL TABLE vt_vector_test USING vectorlite(vector_embedding float32[128] cosine, hnsw(max_elements=100000), '/tmp/test.idx')";
        assert_eq!(extract_index_path(sql), Some("/tmp/test.idx".to_string()));

        let sql_no_path = "CREATE VIRTUAL TABLE vt_vector_test USING vectorlite(vector_embedding float32[128] cosine, hnsw(max_elements=100000))";
        assert_eq!(extract_index_path(sql_no_path), None);
    }
}

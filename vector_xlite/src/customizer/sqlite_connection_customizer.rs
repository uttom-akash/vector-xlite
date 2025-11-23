use r2d2::CustomizeConnection;
use rusqlite::Connection;

use crate::{constant::DEFAULT_SQLITE_TIMEOUT, helper::load_sqlite_vector_extension};

/// Connection customizer for SQLite that loads the vector extension and configures
/// the connection for optimal concurrent access.
#[derive(Debug)]
pub struct SqliteConnectionCustomizer {
    busy_timeout_ms: u32,
}

impl SqliteConnectionCustomizer {
    /// Creates a new customizer with default settings (5 second busy timeout).
    pub fn new() -> Box<Self> {
        Box::new(SqliteConnectionCustomizer {
            busy_timeout_ms: DEFAULT_SQLITE_TIMEOUT,
        })
    }

    /// Creates a new customizer with a custom busy timeout.
    ///
    /// # Arguments
    /// * `busy_timeout_ms` - Timeout in milliseconds to wait when the database is locked.
    ///   Set to 0 to return immediately with SQLITE_BUSY.
    pub fn with_busy_timeout(busy_timeout_ms: u32) -> Box<Self> {
        Box::new(SqliteConnectionCustomizer { busy_timeout_ms })
    }
}

impl Default for SqliteConnectionCustomizer {
    fn default() -> Self {
        SqliteConnectionCustomizer {
            busy_timeout_ms: DEFAULT_SQLITE_TIMEOUT,
        }
    }
}

impl CustomizeConnection<Connection, rusqlite::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // Set busy timeout for better concurrent access handling
        // This makes SQLite wait and retry instead of immediately returning SQLITE_BUSY
        conn.busy_timeout(std::time::Duration::from_millis(self.busy_timeout_ms as u64))?;

        // Load the vector extension
        load_sqlite_vector_extension(conn).map_err(|e| {
            rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(1), Some(e.to_string()))
        })
    }

    fn on_release(&self, _conn: Connection) {}
}

use r2d2::CustomizeConnection;
use rusqlite::Connection;

use crate::helper::load_sqlite_vector_extension;

#[derive(Debug)]
pub struct SqliteConnectionCustomizer;

impl SqliteConnectionCustomizer {
    pub fn new() -> Box<Self> {
        Box::new(SqliteConnectionCustomizer)
    }
}

impl CustomizeConnection<Connection, rusqlite::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        load_sqlite_vector_extension(conn).map_err(|e| {
            rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(1), Some(e.to_string()))
        })
    }

    fn on_release(&self, _conn: Connection) {}
}

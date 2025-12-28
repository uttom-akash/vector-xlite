use rusqlite::Connection;

use crate::error::VecXError;
use crate::vtab::register_hora_module;

/// Loads the vector extension by registering the hora-based virtual table module.
/// This replaces the previous native .so/.dylib/.dll loading with pure Rust.
pub fn load_sqlite_vector_extension(conn: &mut Connection) -> Result<(), VecXError> {
    register_hora_module(conn).map_err(|e| VecXError::ExtensionLoadError(e.to_string()))
}

use rusqlite::{Connection, LoadExtensionGuard, Result};
use std::fs::File;
use std::io::Write;
use std::env;
use std::rc::Rc;

pub fn load_sqlite_vector_extension(conn: Rc<Connection>) -> Result<()> {
    #[cfg(target_os = "linux")]
    const LIB_BYTES: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vectorlite.so"));
    #[cfg(target_os = "macos")]
    const LIB_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/vectorlite.dylib"
    ));
    #[cfg(target_os = "windows")]
    const LIB_BYTES: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/vectorlite.dll"
    ));

    // Write to a temporary file so SQLite can load it
    let tmp_path: std::path::PathBuf = env::temp_dir().join("vectorlite_temp.so");
    File::create(&tmp_path)
        .unwrap()
        .write_all(LIB_BYTES)
        .unwrap();

    unsafe {
        let _guard = LoadExtensionGuard::new(conn.as_ref())?;
        conn.load_extension(tmp_path.to_str().unwrap(), None::<&str>)
    }
}

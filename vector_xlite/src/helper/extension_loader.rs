use rusqlite::{Connection, LoadExtensionGuard};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

use crate::error::VecXError;

static EXT_LOAD_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Default loader that uses the embedded asset bytes (keeps existing API).
pub fn load_sqlite_vector_extension(conn: &mut Connection) -> Result<(), VecXError> {
    // call the more flexible loader with the included bytes and no custom name
    load_sqlite_vector_extension_with_bytes(conn, embedded_lib_bytes())
}

fn create_unique_temp_filename(attempts: u8) -> PathBuf {
    // determine platform extension
    #[cfg(target_os = "linux")]
    let ext = "so";
    #[cfg(target_os = "macos")]
    let ext = "dylib";
    #[cfg(target_os = "windows")]
    let ext = "dll";

    let mut tmp_path = env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let pid = std::process::id();
    let file_name = format!("vectorlite_{}_{}_{}.{}", pid, now, attempts, ext);
    tmp_path.push(file_name);
    tmp_path
}

/// Flexible loader: accepts library bytes and an optional temporary filename.
/// - `lib_bytes`: bytes of the native library (e.g. from include_bytes!)
fn load_sqlite_vector_extension_with_bytes(
    conn: &mut Connection,
    lib_bytes: &[u8],
) -> Result<(), VecXError> {
    // Acquire a process-wide mutex so concurrent threads don't race creating/writing the same file.
    // This keeps the critical section small (write + load).
    let _guard = EXT_LOAD_MUTEX.lock().expect("mutex poisoned");

    let mut attempts = 0u8;
    let mut tmp_path: PathBuf;

    loop {
        tmp_path = create_unique_temp_filename(attempts);

        let open_res = OpenOptions::new()
            .write(true)
            .create_new(true) // atomic create -> fail if exists
            .open(&tmp_path);

        match open_res {
            Ok(mut file) => {
                if let Err(e) = file.write_all(lib_bytes) {
                    return Err(VecXError::from(e));
                }
                if let Err(e) = file.sync_all() {
                    return Err(VecXError::from(e));
                }
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AlreadyExists && attempts < 5 {
                    attempts += 1;
                    continue;
                } else {
                    return Err(VecXError::from(e));
                }
            }
        }
    }

    // load extension using the safe guard
    unsafe {
        let _guard = LoadExtensionGuard::new(conn)?;
        conn.load_extension(
            tmp_path
                .to_str()
                .ok_or_else(|| VecXError::ExtensionLoadError("invalid temporary path".into()))?,
            None::<&str>,
        )?;
    }

    // NOTE: On Unix systems it's safe to remove the file after loading (the loader keeps it open).
    // On Windows the file cannot be deleted while loaded, so keep the file or attempt removal and ignore errors.
    #[cfg(unix)]
    {
        let _ = fs::remove_file(&tmp_path);
    }
    #[cfg(windows)]
    {
        // keep the file; removing may fail while library is loaded
    }

    Ok(())
}

/// Helper returning the compiled-in bytes for the current platform.
fn embedded_lib_bytes() -> &'static [u8] {
    #[cfg(target_os = "linux")]
    {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vectorlite.so"))
    }
    #[cfg(target_os = "macos")]
    {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/vectorlite.dylib"
        ))
    }
    #[cfg(target_os = "windows")]
    {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/vectorlite.dll"
        ))
    }
}

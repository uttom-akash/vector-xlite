# WASM Compilation Plan - Session Save

**Date**: 2025-12-28
**Status**: Planning Phase - Awaiting User Input

## Goal
Compile VectorXLite to WASM with statically linked vectorlite C++ extension.

## Key Findings from Exploration

### 1. Current Extension Loading (`embedded/core/src/helper/extension_loader.rs`)
- Uses `include_bytes!()` to embed .so/.dylib/.dll at compile time
- Dynamically loads via `rusqlite::LoadExtensionGuard` at runtime
- Platform-specific with `#[cfg(target_os = "...")]`
- Entry point: `sqlite3_extension_init()`

### 2. Current Cargo Config (`embedded/core/Cargo.toml`)
```toml
rusqlite = { version = "0.37.0", features = ["bundled", "backup", "load_extension"] }
```
- No existing WASM configuration
- No build.rs in embedded/core

### 3. Vectorlite C++ Library (github.com/1yefuwang1/vectorlite)
- **Build system**: CMake
- **Dependencies**: hnswlib, Google Highway (SIMD), SQLite3, RapidJSON, Abseil, RE2
- **NO existing WASM/Emscripten support**
- **Entry point**: `sqlite3_extension_init(sqlite3 *db, char **pzErrMsg, const sqlite3_api_routines *pApi)`

### 4. Assets Location
```
embedded/core/assets/
├── vectorlite.so      (3.4MB - Linux x86_64)
├── vectorlite.dylib   (2.4MB - macOS x86_64)
└── vectorlite.dll     (1.6MB - Windows x86_64)
```

## Open Questions (Need User Input)

### Q1: SIMD Handling
Vectorlite uses Google Highway for SIMD acceleration. Options:
- **A) Disable SIMD for WASM** - Compile without Highway. Slower but simpler.
- **B) Try WASM SIMD128** - May require significant vectorlite modifications.
- **C) Use pure-Rust HNSW** - Replace with `instant-distance` crate for WASM only.

### Q2: Code Organization
- **A) Same crate, cfg flags** - Add `#[cfg(target_arch = "wasm32")]` to embedded/core
- **B) Separate wasm crate** - Create `embedded/wasm/` with WASM-specific code

## Proposed Architecture

```
Native Build (Linux/macOS/Windows):
  cargo build --release
  → Dynamic loading of .so/.dylib/.dll (current behavior)

WASM Build:
  cargo build --target wasm32-unknown-unknown
  → Static linking of vectorlite.a compiled via Emscripten
  → Or pure-Rust HNSW alternative
```

## Files to Modify

1. `embedded/core/Cargo.toml` - Add WASM target config, conditional deps
2. `embedded/core/build.rs` - Create for WASM static linking
3. `embedded/core/src/helper/extension_loader.rs` - Add WASM conditional compilation
4. `embedded/core/src/customizer/sqlite_connection_customizer.rs` - WASM path

## Next Steps

1. User decides on SIMD handling approach
2. User decides on code organization
3. If using vectorlite for WASM:
   - Set up Emscripten toolchain
   - Create CMake config to build vectorlite.a for WASM
   - Modify build.rs to link static library
4. Implement conditional compilation in Rust code
5. Test both native and WASM builds

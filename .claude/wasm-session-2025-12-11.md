# WASM Compilation Session - 2025-12-11

## Objective
Compile `/home/akash/Dev/vector-db-rs/embedded` to WebAssembly and integrate into browser.

## Current State Analysis

### Project Structure
- **Location**: `/home/akash/Dev/vector-db-rs/embedded/core`
- **Package**: `vector_xlite` v1.4.0
- **Language**: Rust (Edition 2021)
- **Current Branch**: `wasm`

### Core Dependencies (Cargo.toml)
```toml
rusqlite = { version = "0.37.0", features = ["load_extension", "backup"] }
regex = "1.12.2"
once_cell = "1.21.3"
r2d2 = "0.8.10"
r2d2_sqlite = { version = "0.31.0"}
```

### Main API Surface (vector_xlite.rs)
```rust
pub struct VectorXLite {
    query_planner: Box<dyn QueryPlanner>,
    query_executor: Box<dyn QueryExecutor>,
}

// Public Methods:
- new(connection_pool: Pool<SqliteConnectionManager>) -> Result<VectorXLite, VecXError>
- create_collection(&self, collection_config: CollectionConfig) -> Result<(), VecXError>
- insert(&self, create_point: InsertPoint) -> Result<(), VecXError>
- search(&self, search_point: SearchPoint) -> Result<Vec<HashMap<String, String>>, VecXError>
- collection_exists(&self, collection_name: &str) -> Result<bool, VecXError>
- delete(&self, delete_point: DeletePoint) -> Result<(), VecXError>
- delete_collection(&self, delete_collection: DeleteCollection) -> Result<(), VecXError>
```

## WASM Feasibility: ✓ YES

### Challenges Identified
1. **libsqlite3-sys**: Native SQLite binding won't work in WASM
   - Solution: Use `bundled` feature to compile SQLite to WASM

2. **Connection Pooling (r2d2/r2d2_sqlite)**: Less relevant in browser
   - Solution: Use single connection or simplified pooling for WASM target

3. **File I/O**: Browser has different storage model
   - Solution: Use WASM-compatible storage (IndexedDB via JS, or in-memory)

4. **Threading**: WASM has limited threading support
   - Solution: Single-threaded model with async/await

### Proposed Solution

#### Approach Options Discussed:
1. Create new `wasm` module in embedded directory with WASM-specific bindings
2. Modify existing core to support both native and WASM targets (conditional compilation)
3. Documentation-only approach for manual implementation

**Status**: Awaiting user decision on preferred approach

## Implementation Plan (TODO List)

- [x] Analyze current dependencies and WASM compatibility
- [ ] Create WASM-specific Cargo.toml with bundled SQLite
- [ ] Add conditional compilation for WASM vs native
- [ ] Create wasm-bindgen wrapper with JS-friendly API
- [ ] Set up build script for WASM compilation
- [ ] Create browser example with HTML/JS integration
- [ ] Test WASM build and browser integration

## Technical Requirements for WASM Build

### Additional Dependencies Needed
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["Window", "Storage"] }
console_error_panic_hook = "0.1"

[dependencies]
rusqlite = { version = "0.37.0", features = ["bundled"] }  # bundled SQLite
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

### Build Commands
```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir pkg

# Build for nodejs
wasm-pack build --target nodejs --out-dir pkg-node

# Build for bundlers (webpack, rollup, etc)
wasm-pack build --target bundler --out-dir pkg-bundler
```

## Expected JavaScript API

```javascript
import init, { VectorXLite } from './vector_xlite.js';

// Initialize WASM module
await init();

// Create database instance
const db = new VectorXLite();

// Create collection
await db.createCollection({
  name: "documents",
  dimensions: 384,
  distance_metric: "cosine"
});

// Insert vectors
await db.insert({
  collection: "documents",
  vector: [0.1, 0.2, ...], // 384 dimensions
  payload: { title: "Doc 1", content: "..." }
});

// Search
const results = await db.search({
  collection: "documents",
  vector: [0.1, 0.2, ...],
  limit: 10
});

// Check if collection exists
const exists = await db.collectionExists("documents");

// Delete points
await db.delete({
  collection: "documents",
  ids: ["id1", "id2"]
});

// Delete collection
await db.deleteCollection("documents");
```

## Next Steps

1. **User Decision Required**: Choose implementation approach
   - Option A: New wasm module (isolated, cleaner)
   - Option B: Modify existing core (unified codebase)
   - Option C: Documentation only

2. **Once Approach Selected**:
   - Configure Cargo.toml for WASM target
   - Add wasm-bindgen bindings
   - Handle connection management for WASM
   - Create build scripts
   - Create browser example
   - Test in browser environment

## File References
- Main library: `/home/akash/Dev/vector-db-rs/embedded/core/src/lib.rs`
- Core implementation: `/home/akash/Dev/vector-db-rs/embedded/core/src/vector_xlite.rs`
- Cargo config: `/home/akash/Dev/vector-db-rs/embedded/core/Cargo.toml`
- Source files: 33 Rust files in `/home/akash/Dev/vector-db-rs/embedded/core/src`

## Notes
- Current git status: clean
- Recent commit: `aeabd2b [ VXLite 23 ] : Add delete API (#43)`
- Repository: https://github.com/uttom-akash/vector-xlite
- License: MIT OR Apache-2.0

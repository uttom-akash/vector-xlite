# VectorXLite WASM Compilation Guide

**Project**: VectorXLite Embedded
**Version**: 1.4.0
**Target**: WebAssembly with JavaScript Glue Code
**Last Updated**: 2025-12-15


## Notes:
- Use in-memory or IndexedDB/OPFS for persistence
---

## Table of Contents

1. [Project Analysis](#project-analysis)
2. [Prerequisites](#prerequisites)
3. [Critical Challenges](#critical-challenges)
4. [Step-by-Step Compilation](#step-by-step-compilation)
5. [Usage Examples](#usage-examples)
6. [Build Configurations](#build-configurations)
7. [Troubleshooting](#troubleshooting)
8. [Production Considerations](#production-considerations)

---

## Project Analysis

### Project Details

**Location**: `/home/akash/Dev/vector-db-rs/embedded/core`
**Language**: Rust 2021 Edition
**Package**: `vector_xlite` v1.4.0

### Current Dependencies

```toml
rusqlite = { version = "0.37.0", features = ["load_extension", "backup"] }
regex = "1.12.2"
once_cell = "1.21.3"
r2d2 = "0.8.10"
r2d2_sqlite = { version = "0.31.0"}
```

### Public API Surface

The main `VectorXLite` struct provides these methods:
- `new(connection_pool: Pool<SqliteConnectionManager>)` - Initialize database
- `create_collection(config: CollectionConfig)` - Create vector collection
- `insert(point: InsertPoint)` - Insert vectors with payload
- `search(query: SearchPoint)` - Similarity search
- `collection_exists(name: &str)` - Check collection existence
- `delete(point: DeletePoint)` - Delete vectors
- `delete_collection(collection: DeleteCollection)` - Delete collection

---

## Prerequisites

### Required Tools

```bash
# Install Rust toolchain
rustup install stable

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (recommended)
cargo install wasm-pack

# Install wasm-bindgen-cli (for manual builds)
cargo install wasm-bindgen-cli

# Optional: WASM optimizer
cargo install wasm-opt
```

### Verify Installation

```bash
rustc --version
wasm-pack --version
wasm-bindgen --version
```

---

## Critical Challenges

### 1. Native SQLite Extension Loading

**Issue**: `extension_loader.rs` loads platform-specific native libraries (`.so/.dylib/.dll`) from the filesystem, which is incompatible with browser environments.

**Location**: `/embedded/core/src/helper/extension_loader.rs:16-128`

**Current Implementation**:
```rust
fn load_sqlite_vector_extension(conn: &mut Connection) -> Result<(), VecXError> {
    // Writes native library to temp directory
    // Uses LoadExtensionGuard (unsafe FFI)
    // Loads .so/.dylib/.dll file
}
```

**WASM Solutions**:
- **Option A**: Compile vectorlite extension to WASM (complex, requires C to WASM compilation)
- **Option B**: Use pure-Rust HNSW implementation (recommended)
- **Option C**: Disable extension for WASM builds (limited functionality)

### 2. Connection Pooling

**Issue**: `r2d2` connection pooling assumes multi-threaded environment.

**Solution**: Use single connection or simplified pooling for WASM target.

### 3. File System Operations

**Issue**: Uses `std::fs`, `std::env::temp_dir()` for extension loading.

**Solution**: WASM uses virtual filesystem or in-memory storage.

### 4. Threading Model

**Issue**: WASM is single-threaded by default.

**Solution**: Use Web Workers with SharedArrayBuffer for parallelism (optional).

---

## Step-by-Step Compilation

### Step 1: Create WASM-Specific Cargo Configuration

Create `/embedded/core/Cargo-wasm.toml`:

```toml
[package]
name = "vector_xlite_wasm"
version = "1.4.0"
edition = "2021"
authors = ["Uttom Akash <uttom.akash71@gmail.com>"]
description = "VectorXLite WASM bindings for browser usage"
license = "MIT OR Apache-2.0"
repository = "https://github.com/uttom-akash/vector-xlite"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Core dependencies with WASM-compatible features
rusqlite = { version = "0.37.0", features = ["bundled", "backup"] }
regex = "1.12.2"
once_cell = "1.21.3"

# WASM-specific dependencies
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
js-sys = "0.3"
console_error_panic_hook = "0.1"
getrandom = { version = "0.2", features = ["js"] }

# Web APIs
web-sys = { version = "0.3", features = [
    "Window",
    "Storage",
    "console",
] }

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip debug symbols

[profile.release.package."*"]
opt-level = "z"      # Optimize dependencies for size
```

### Step 2: Create WASM Bindings Module

Create `/embedded/core/src/wasm.rs`:

```rust
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Set panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct WasmCollectionConfig {
    pub collection_name: String,
    pub vector_dimension: u32,
    pub distance: String,  // "Cosine", "L2", or "IP"
    pub payload_schema: Option<String>,
    pub max_elements: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmInsertPoint {
    pub collection_name: String,
    pub id: u64,
    pub vector: Vec<f32>,
    pub payload_query: String,
}

#[derive(Serialize, Deserialize)]
pub struct WasmSearchPoint {
    pub collection_name: String,
    pub vector: Vec<f32>,
    pub top_k: usize,
    pub payload_query: Option<String>,
}

#[wasm_bindgen]
pub struct VectorXLiteWasm {
    // Internal implementation
    // Note: Will need to refactor connection management for WASM
}

#[wasm_bindgen]
impl VectorXLiteWasm {
    /// Create a new VectorXLite instance
    ///
    /// For WASM, this uses an in-memory SQLite database
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<VectorXLiteWasm, JsValue> {
        // TODO: Initialize with single connection (no r2d2 pool)
        // Use bundled SQLite with in-memory database
        Err(JsValue::from_str("Not yet fully implemented - requires connection refactoring"))
    }

    /// Create a new collection
    #[wasm_bindgen(js_name = createCollection)]
    pub async fn create_collection(&self, config: JsValue) -> Result<(), JsValue> {
        let config: WasmCollectionConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

        // TODO: Call internal create_collection logic
        Ok(())
    }

    /// Insert a vector with payload
    #[wasm_bindgen(js_name = insert)]
    pub async fn insert(&self, data: JsValue) -> Result<(), JsValue> {
        let point: WasmInsertPoint = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Invalid insert point: {}", e)))?;

        // TODO: Call internal insert logic
        Ok(())
    }

    /// Search for similar vectors
    #[wasm_bindgen(js_name = search)]
    pub async fn search(&self, query: JsValue) -> Result<JsValue, JsValue> {
        let search: WasmSearchPoint = serde_wasm_bindgen::from_value(query)
            .map_err(|e| JsValue::from_str(&format!("Invalid search point: {}", e)))?;

        // TODO: Call internal search logic
        let results: Vec<HashMap<String, String>> = Vec::new();
        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Check if a collection exists
    #[wasm_bindgen(js_name = collectionExists)]
    pub async fn collection_exists(&self, name: String) -> Result<bool, JsValue> {
        // TODO: Call internal collection_exists logic
        Ok(false)
    }

    /// Delete vectors from a collection
    #[wasm_bindgen(js_name = delete)]
    pub async fn delete(&self, data: JsValue) -> Result<(), JsValue> {
        // TODO: Implement delete logic
        Ok(())
    }

    /// Delete an entire collection
    #[wasm_bindgen(js_name = deleteCollection)]
    pub async fn delete_collection(&self, name: String) -> Result<(), JsValue> {
        // TODO: Implement delete collection logic
        Ok(())
    }
}
```

### Step 3: Add Conditional Compilation for Native Code

Modify `/embedded/core/src/customizer/sqlite_connection_customizer.rs`:

```rust
impl CustomizeConnection<Connection, rusqlite::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.busy_timeout(std::time::Duration::from_millis(self.busy_timeout_ms as u64))?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native configuration
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;

            // Load the vector extension (native only)
            load_sqlite_vector_extension(conn).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(e.to_string())
                )
            })?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM-specific configuration
            // Note: WAL mode is not supported in WASM
            conn.pragma_update(None, "journal_mode", "MEMORY")?;

            // TODO: Initialize pure-Rust vector search implementation
            // or use alternative approach without native extension
        }

        Ok(())
    }

    fn on_release(&self, _conn: Connection) {}
}
```

### Step 4: Update lib.rs

Add WASM module to `/embedded/core/src/lib.rs`:

```rust
mod executor;
mod helper;
mod planner;
pub mod types;
mod vector_xlite;
mod constant;
pub mod error;
pub mod customizer;
pub mod snapshot;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use vector_xlite::*;
```

### Step 5: Build Commands

#### Using wasm-pack (Recommended)

```bash
# Navigate to project directory
cd /home/akash/Dev/vector-db-rs/embedded/core

# Build for web browsers (ES modules)
wasm-pack build --target web --out-dir ../../wasm-pkg/web

# Build for Node.js
wasm-pack build --target nodejs --out-dir ../../wasm-pkg/node

# Build for bundlers (webpack, rollup, vite, parcel)
wasm-pack build --target bundler --out-dir ../../wasm-pkg/bundler

# Build with optimizations (production)
wasm-pack build --target web --out-dir ../../wasm-pkg/web --release

# Build with debug info (development)
wasm-pack build --target web --out-dir ../../wasm-pkg/web --dev

# Build with profiling enabled
wasm-pack build --target web --out-dir ../../wasm-pkg/web --profiling
```

#### Manual Build (Advanced)

```bash
# Build the WASM binary
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings with wasm-bindgen
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vector_xlite_wasm.wasm \
  --out-dir ./wasm-output \
  --target web \
  --no-typescript

# With TypeScript definitions
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vector_xlite_wasm.wasm \
  --out-dir ./wasm-output \
  --target web

# Optimize with wasm-opt
wasm-opt -Os \
  wasm-output/vector_xlite_wasm_bg.wasm \
  -o wasm-output/vector_xlite_wasm_optimized.wasm
```

**Output files**:
- `vector_xlite_wasm.js` - JavaScript glue code
- `vector_xlite_wasm_bg.wasm` - WebAssembly binary
- `vector_xlite_wasm.d.ts` - TypeScript type definitions
- `package.json` - NPM package metadata

---

## Usage Examples

### Browser Usage (Vanilla JavaScript)

Create `index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VectorXLite WASM Demo</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        #output {
            background: #f5f5f5;
            padding: 20px;
            border-radius: 5px;
            white-space: pre-wrap;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <h1>VectorXLite WASM Demo</h1>
    <button id="runDemo">Run Demo</button>
    <div id="output">Click "Run Demo" to start...</div>

    <script type="module">
        import init, { VectorXLiteWasm } from './wasm-pkg/web/vector_xlite_wasm.js';

        async function runDemo() {
            const output = document.getElementById('output');

            try {
                output.textContent = 'Initializing WASM module...\n';

                // Initialize WASM
                await init();
                output.textContent += 'WASM initialized ✓\n\n';

                // Create database instance
                output.textContent += 'Creating database instance...\n';
                const db = new VectorXLiteWasm();
                output.textContent += 'Database created ✓\n\n';

                // Create collection
                output.textContent += 'Creating collection...\n';
                await db.createCollection({
                    collection_name: "documents",
                    vector_dimension: 384,
                    distance: "Cosine",
                    payload_schema: `CREATE TABLE documents (
                        rowid INTEGER PRIMARY KEY,
                        title TEXT NOT NULL,
                        content TEXT,
                        category TEXT
                    )`,
                    max_elements: 10000
                });
                output.textContent += 'Collection created ✓\n\n';

                // Generate sample vector (384 dimensions)
                const vector = new Array(384).fill(0).map(() => Math.random() - 0.5);

                // Insert vector
                output.textContent += 'Inserting vector...\n';
                await db.insert({
                    collection_name: "documents",
                    id: 1,
                    vector: vector,
                    payload_query: `INSERT INTO documents(rowid, title, content, category)
                                   VALUES (?1, 'Sample Document', 'This is a test', 'general')`
                });
                output.textContent += 'Vector inserted ✓\n\n';

                // Check if collection exists
                output.textContent += 'Checking collection existence...\n';
                const exists = await db.collectionExists("documents");
                output.textContent += `Collection exists: ${exists} ✓\n\n`;

                // Search
                output.textContent += 'Searching for similar vectors...\n';
                const queryVector = new Array(384).fill(0).map(() => Math.random() - 0.5);
                const results = await db.search({
                    collection_name: "documents",
                    vector: queryVector,
                    top_k: 10,
                    payload_query: "SELECT * FROM documents WHERE category = 'general'"
                });

                output.textContent += 'Search results:\n';
                output.textContent += JSON.stringify(results, null, 2);

            } catch (error) {
                output.textContent += `\n\nError: ${error.message}\n${error.stack}`;
                console.error('Demo failed:', error);
            }
        }

        document.getElementById('runDemo').addEventListener('click', runDemo);
    </script>
</body>
</html>
```

### Usage with Vite

**Install dependencies**:
```bash
npm install vite vite-plugin-wasm
```

**vite.config.js**:
```javascript
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [
    wasm(),
    topLevelAwait()
  ],
  optimizeDeps: {
    exclude: ['vector_xlite_wasm']
  }
});
```

**main.js**:
```javascript
import init, { VectorXLiteWasm } from 'vector_xlite_wasm';

async function main() {
  // Initialize WASM module
  await init();

  // Create database
  const db = new VectorXLiteWasm();

  // Create collection with 768-dimensional vectors (e.g., BERT embeddings)
  await db.createCollection({
    collection_name: "embeddings",
    vector_dimension: 768,
    distance: "Cosine",
    payload_schema: `
      CREATE TABLE embeddings (
        rowid INTEGER PRIMARY KEY,
        text TEXT NOT NULL,
        metadata JSON
      )
    `
  });

  // Insert embeddings
  const embedding = new Array(768).fill(0.1);
  await db.insert({
    collection_name: "embeddings",
    id: 1,
    vector: embedding,
    payload_query: `
      INSERT INTO embeddings(rowid, text, metadata)
      VALUES (?1, 'Sample text', '{"source": "user_input"}')
    `
  });

  // Search
  const queryEmbedding = new Array(768).fill(0.12);
  const results = await db.search({
    collection_name: "embeddings",
    vector: queryEmbedding,
    top_k: 5,
    payload_query: "SELECT rowid, text FROM embeddings"
  });

  console.log('Search results:', results);
}

main().catch(console.error);
```

### Usage with React

```jsx
import React, { useEffect, useState } from 'react';
import init, { VectorXLiteWasm } from 'vector_xlite_wasm';

function App() {
  const [db, setDb] = useState(null);
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function initWasm() {
      try {
        await init();
        const database = new VectorXLiteWasm();

        // Initialize collection
        await database.createCollection({
          collection_name: "items",
          vector_dimension: 128,
          distance: "Cosine",
          payload_schema: `
            CREATE TABLE items (
              rowid INTEGER PRIMARY KEY,
              name TEXT NOT NULL
            )
          `
        });

        setDb(database);
        setLoading(false);
      } catch (error) {
        console.error('WASM initialization failed:', error);
        setLoading(false);
      }
    }

    initWasm();
  }, []);

  const handleSearch = async () => {
    if (!db) return;

    const queryVector = new Array(128).fill(0).map(() => Math.random());
    const searchResults = await db.search({
      collection_name: "items",
      vector: queryVector,
      top_k: 10,
      payload_query: "SELECT * FROM items"
    });

    setResults(searchResults);
  };

  if (loading) return <div>Loading WASM module...</div>;

  return (
    <div>
      <h1>Vector Search Demo</h1>
      <button onClick={handleSearch}>Search</button>
      <pre>{JSON.stringify(results, null, 2)}</pre>
    </div>
  );
}

export default App;
```

### Node.js Usage

```javascript
const { VectorXLiteWasm } = require('./wasm-pkg/node/vector_xlite_wasm.js');

async function main() {
  // Create database (WASM is auto-initialized in Node.js target)
  const db = new VectorXLiteWasm();

  // Create collection
  await db.createCollection({
    collection_name: "vectors",
    vector_dimension: 256,
    distance: "L2"
  });

  // Insert
  await db.insert({
    collection_name: "vectors",
    id: 1,
    vector: new Array(256).fill(0.5),
    payload_query: "INSERT INTO vectors(rowid) VALUES (?1)"
  });

  // Search
  const results = await db.search({
    collection_name: "vectors",
    vector: new Array(256).fill(0.6),
    top_k: 10
  });

  console.log(results);
}

main();
```

---

## Build Configurations

### Development Build

```bash
# Fast compilation, larger binary, includes debug info
wasm-pack build --target web --dev --out-dir ./pkg-dev

# Features:
# - Faster compile times
# - Debug symbols included
# - No optimizations
# - Larger file size (~5-10MB)
```

### Production Build

```bash
# Optimized for size and performance
wasm-pack build --target web --release --out-dir ./pkg

# Then optimize further with wasm-opt
wasm-opt -Os pkg/vector_xlite_wasm_bg.wasm -o pkg/optimized.wasm

# Features:
# - Maximum optimizations
# - Minimal file size (~2-4MB after gzip)
# - No debug info
# - Slower compile times
```

### Profile Build

```bash
# For performance profiling
wasm-pack build --target web --profiling --out-dir ./pkg-profile

# Use with browser DevTools Performance tab
# Features:
# - Optimized but with symbol names preserved
# - Good for profiling performance bottlenecks
```

### Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"     # Optimize aggressively for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Single codegen unit for better optimization
panic = 'abort'     # Don't include unwinding code
strip = true        # Strip symbols

[profile.release.build-override]
opt-level = 0       # Don't optimize build scripts
```

**Expected sizes**:
- Unoptimized debug: ~8-12 MB
- Release build: ~3-5 MB
- Release + wasm-opt: ~2-4 MB
- gzip compressed: ~800KB-1.5MB
- brotli compressed: ~600KB-1.2MB

---

## Troubleshooting

### Error: "Cannot find bundled SQLite"

**Cause**: The `bundled` feature is not enabled for rusqlite.

**Solution**:
```toml
# In Cargo.toml
rusqlite = { version = "0.37.0", features = ["bundled"] }
```

Then clean and rebuild:
```bash
cargo clean
wasm-pack build --target web
```

### Error: "getrandom: this target is not supported"

**Cause**: The `getrandom` crate needs JS support for WASM.

**Solution**:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

### Error: "Memory access out of bounds"

**Cause**: WASM memory limit exceeded.

**Solution**: Increase initial memory in JavaScript:
```javascript
import init from './vector_xlite_wasm.js';

await init({
  memory: new WebAssembly.Memory({
    initial: 256,  // 256 * 64KB = 16MB
    maximum: 512,  // 512 * 64KB = 32MB
    shared: false
  })
});
```

### Error: "Failed to load extension"

**Cause**: Trying to load native extension in WASM environment.

**Solution**: Add conditional compilation:
```rust
#[cfg(not(target_arch = "wasm32"))]
load_sqlite_vector_extension(conn)?;
```

### Error: "Cannot use WAL journal mode"

**Cause**: SQLite's Write-Ahead Logging is not supported in WASM.

**Solution**:
```rust
#[cfg(target_arch = "wasm32")]
conn.pragma_update(None, "journal_mode", "MEMORY")?;

#[cfg(not(target_arch = "wasm32"))]
conn.pragma_update(None, "journal_mode", "WAL")?;
```

### Error: "File size too large"

**Cause**: WASM binary exceeds reasonable size limits.

**Solutions**:
1. Use aggressive optimization:
   ```bash
   wasm-opt -Oz input.wasm -o output.wasm
   ```

2. Enable compression on web server:
   ```nginx
   # nginx config
   gzip on;
   gzip_types application/wasm;
   ```

3. Split code with dynamic imports:
   ```javascript
   const wasmModule = await import('./heavy-module.js');
   ```

### Issue: Slow Performance

**Causes & Solutions**:

1. **Not using release build**:
   ```bash
   wasm-pack build --release
   ```

2. **Browser not optimized**:
   - Use Chrome/Edge with WASM optimizations enabled
   - Check `chrome://flags/#enable-webassembly-baseline`

3. **Too many allocations**:
   - Reuse vectors instead of creating new ones
   - Use typed arrays (`Float32Array`) instead of regular arrays

4. **Blocking main thread**:
   - Move WASM work to Web Worker
   - Use async operations

---

## Production Considerations

### 1. Vector Extension Dependency

**Critical Issue**: The native `vectorlite` extension cannot be loaded in WASM.

**Solutions**:

#### Option A: Pure-Rust HNSW Implementation (Recommended)

Use a pure-Rust implementation like `instant-distance` or `hnswlib-rs`:

```toml
[dependencies]
instant-distance = "0.6"  # Pure Rust, WASM-compatible
```

Replace the native extension with Rust implementation in WASM builds.

#### Option B: Compile Extension to WASM (Complex)

Compile the C-based vectorlite extension using Emscripten:
```bash
emcc vectorlite.c -o vectorlite.wasm
```

This requires significant effort and may have performance implications.

#### Option C: Server-Side Hybrid (Practical)

Keep vector operations on the server, use WASM for:
- Client-side data preparation
- Local caching
- Offline functionality

### 2. Storage and Persistence

**In-Memory Only** (Default):
```javascript
// Data lost on page reload
const db = new VectorXLiteWasm();
```

**IndexedDB Persistence**:
```javascript
// Use sql.js-httpvfs or similar
import { createDbWorker } from 'sql.js-httpvfs';

const worker = await createDbWorker(
  [{ from: 'inline', config: { serverMode: 'full' } }],
  '/path/to/sqlite.worker.js',
  '/path/to/sqlite.wasm'
);
```

**OPFS (Origin Private File System)**:
```javascript
// Chrome 86+, modern browsers
const root = await navigator.storage.getDirectory();
const fileHandle = await root.getFileHandle('vector.db', { create: true });
```

### 3. Bundle Size Optimization

**Compression**:
```nginx
# Nginx
gzip on;
gzip_types application/wasm application/javascript;
gzip_min_length 1024;

# Or use Brotli (better compression)
brotli on;
brotli_types application/wasm application/javascript;
```

**Code Splitting**:
```javascript
// Load WASM on-demand
const loadVectorDB = async () => {
  const { default: init, VectorXLiteWasm } = await import('./vector_xlite_wasm.js');
  await init();
  return VectorXLiteWasm;
};
```

**CDN Delivery**:
```html
<script type="module">
  import init from 'https://cdn.example.com/vector_xlite_wasm.js';
</script>
```

### 4. Performance Optimization

**Use Typed Arrays**:
```javascript
// Good: Efficient memory usage
const vector = new Float32Array(384);
for (let i = 0; i < 384; i++) {
  vector[i] = Math.random();
}

// Bad: Less efficient
const vector = Array(384).fill(0).map(() => Math.random());
```

**Batch Operations**:
```javascript
// Good: Single WASM call
await db.insertBatch(points);

// Bad: Multiple WASM calls
for (const point of points) {
  await db.insert(point);
}
```

**Web Workers**:
```javascript
// vector-worker.js
import init, { VectorXLiteWasm } from './vector_xlite_wasm.js';

let db;

self.onmessage = async (e) => {
  if (!db) {
    await init();
    db = new VectorXLiteWasm();
  }

  const { action, payload } = e.data;

  switch (action) {
    case 'search':
      const results = await db.search(payload);
      self.postMessage({ results });
      break;
  }
};
```

### 5. Error Handling

```javascript
try {
  await init();
  const db = new VectorXLiteWasm();

  // Operations...

} catch (error) {
  if (error.message.includes('memory')) {
    console.error('WASM memory limit exceeded');
    // Fallback to server-side processing
  } else if (error.message.includes('compilation')) {
    console.error('WASM not supported in this browser');
    // Show compatibility warning
  } else {
    console.error('Unknown error:', error);
    // Generic error handling
  }
}
```

### 6. Browser Compatibility

**Minimum Requirements**:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

**Feature Detection**:
```javascript
function supportsWasm() {
  try {
    if (typeof WebAssembly === 'object' &&
        typeof WebAssembly.instantiate === 'function') {
      const module = new WebAssembly.Module(
        Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00)
      );
      if (module instanceof WebAssembly.Module) {
        return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
      }
    }
  } catch (e) {}
  return false;
}

if (!supportsWasm()) {
  alert('Your browser does not support WebAssembly');
}
```

### 7. Security Considerations

**Content Security Policy**:
```html
<meta http-equiv="Content-Security-Policy"
      content="script-src 'self' 'wasm-unsafe-eval'">
```

**Same-Origin Policy**:
```javascript
// WASM must be served from same origin or with CORS headers
fetch('/vector_xlite_wasm_bg.wasm', {
  mode: 'same-origin',
  credentials: 'omit'
});
```

**Data Validation**:
```javascript
function validateVector(vector, expectedDim) {
  if (!Array.isArray(vector) && !(vector instanceof Float32Array)) {
    throw new Error('Vector must be an array or Float32Array');
  }
  if (vector.length !== expectedDim) {
    throw new Error(`Expected ${expectedDim} dimensions, got ${vector.length}`);
  }
  if (vector.some(v => !isFinite(v))) {
    throw new Error('Vector contains invalid values');
  }
  return true;
}
```

---

## Key Differences: Native vs WASM

| Feature | Native | WASM |
|---------|--------|------|
| **Extension Loading** | Loads `.so/.dylib/.dll` from filesystem | Must use bundled SQLite or pure-Rust implementation |
| **Connection Pooling** | r2d2 multi-threaded pooling | Single connection or simplified pooling |
| **File I/O** | Direct filesystem access | Virtual filesystem, IndexedDB, or in-memory only |
| **Threading** | Multi-threaded with Rayon/std::thread | Single-threaded (or Web Workers with SharedArrayBuffer) |
| **Journal Mode** | WAL (Write-Ahead Log) | MEMORY or DELETE (WAL not supported) |
| **Performance** | Native speed | ~70-90% of native performance |
| **Binary Size** | Not a concern | Critical (aim for <2MB compressed) |
| **Startup Time** | Instant | ~100-500ms for WASM compilation |
| **Memory** | System RAM | Browser-imposed limits (~2GB) |
| **Distribution** | Compile for each platform | Single WASM binary for all platforms |

---

## Deployment Checklist

- [ ] WASM binary is optimized with `wasm-opt`
- [ ] gzip/brotli compression enabled on web server
- [ ] CORS headers configured if serving from CDN
- [ ] Feature detection implemented for browser compatibility
- [ ] Error handling for WASM initialization failures
- [ ] Memory limits configured appropriately
- [ ] Web Workers implemented for heavy operations
- [ ] TypeScript definitions included (if using TypeScript)
- [ ] Source maps generated for debugging
- [ ] Performance profiling completed
- [ ] Bundle size analysis performed
- [ ] Security headers (CSP) configured
- [ ] Fallback strategy for unsupported browsers
- [ ] Loading indicators for WASM initialization

---

## References

### Documentation
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [SQLite WASM](https://sqlite.org/wasm/)

### Tools
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- [wasm-opt](https://github.com/WebAssembly/binaryen)

### Related Projects
- [sql.js](https://github.com/sql-js/sql.js/) - SQLite compiled to WASM
- [rusqlite](https://github.com/rusqlite/rusqlite) - Rust SQLite bindings
- [instant-distance](https://github.com/InstantDomain/instant-distance) - Pure Rust HNSW

---

## Next Steps

1. **Resolve Native Extension Dependency**: Implement or integrate a pure-Rust HNSW library
2. **Refactor Connection Management**: Create WASM-specific connection handling without r2d2
3. **Implement WASM Bindings**: Complete the `wasm.rs` module implementation
4. **Create Test Suite**: Add wasm-bindgen-test cases
5. **Build Example Application**: Create a working demo in `examples/wasm`
6. **Performance Benchmarking**: Compare WASM vs Native performance
7. **Documentation**: Add JSDoc comments to generated JavaScript
8. **Publishing**: Publish to npm registry

---

## Support

For issues and questions:
- GitHub Issues: https://github.com/uttom-akash/vector-xlite/issues
- Documentation: https://docs.rs/vector_xlite

---

**Last Updated**: 2025-12-15
**Author**: VectorXLite Team
**License**: MIT OR Apache-2.0

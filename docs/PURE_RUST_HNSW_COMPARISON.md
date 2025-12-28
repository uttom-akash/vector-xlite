# Pure-Rust HNSW Library Comparison

Evaluation of pure-Rust HNSW libraries as potential replacements for `vectorlite.so` to enable WASM compilation.

## Current Setup: vectorlite.so

| Attribute | Value |
|-----------|-------|
| Language | C++ with SQLite extension interface |
| Dependencies | hnswlib, Google Highway (SIMD), RapidJSON, Abseil, RE2 |
| Size | 3.4MB (Linux), 2.4MB (macOS), 1.6MB (Windows) |
| WASM Support | None |
| Loading | Dynamic via `sqlite3_extension_init()` |

**Problem**: Native `.so/.dylib/.dll` files cannot be loaded in WASM environments. Need pure-Rust alternative.

---

## Pure-Rust HNSW Libraries

### Comparison Table

| Library | Stars | License | Last Active | WASM | no_std | Serde | Distance Metrics |
|---------|-------|---------|-------------|------|--------|-------|------------------|
| **hora** | 2.7k | Apache-2.0 | Active | Yes | No | No | L2, Cosine, Dot, Manhattan |
| **instant-distance** | 338 | Apache-2.0 | Active | ? | No | No | Custom via trait |
| **rust-cv/hnsw** | 251 | MIT | Active | ? | **Yes** | **Yes** | Custom via `space` trait |
| **hnswlib-rs** | 222 | Apache-2.0/MIT | Jul 2024 | ? | No | **Yes** | L1, L2, Cosine, Jaccard, Hamming, Hellinger, JS |
| **granne** | 320 | MIT | Jun 2021 | No | No | No | Cosine only |

### Detailed Analysis

---

#### 1. hora (Recommended for WASM)

**Repository**: https://github.com/hora-search/hora

| Attribute | Value |
|-----------|-------|
| Stars | 2,700+ |
| License | Apache-2.0 |
| Pure Rust | Yes |
| WASM Support | **Official** (`horajs` on NPM) |
| SIMD | Yes (accelerated) |

**Pros**:
- Proven WASM support with official JavaScript bindings
- Multiple index types: HNSW, SSG, PQIVF, RPT, BruteForce
- No heavy dependencies (no BLAS)
- Multi-language bindings (Python, JS, Java)
- Active development

**Cons**:
- No `no_std` support
- No serde serialization
- Larger feature set than needed (may increase binary size)

**Distance Metrics**: Euclidean, Cosine, Dot Product, Manhattan

---

#### 2. rust-cv/hnsw (Recommended for no_std/embedded)

**Repository**: https://github.com/rust-cv/hnsw

| Attribute | Value |
|-----------|-------|
| Stars | 251 |
| License | MIT |
| Pure Rust | Yes |
| no_std | **Yes** |
| Serde | **Yes** (feature flag) |

**Pros**:
- `no_std` support for embedded/WASM environments
- Serde serialization for index persistence
- SIMD-capable types via `space` crate
- Clean, focused implementation
- MIT license (permissive)

**Cons**:
- Smaller community than hora
- No official WASM examples
- Distance metrics via trait (need to implement)

**Distance Metrics**: Hamming, Euclidean, custom via `Space` trait

---

#### 3. instant-distance (Production-Proven)

**Repository**: https://github.com/InstantDomainSearch/instant-distance

| Attribute | Value |
|-----------|-------|
| Stars | 338 |
| License | Apache-2.0 |
| Pure Rust | Yes |
| Production Use | **InstantDomainSearch backend** |

**Pros**:
- Battle-tested in production
- Python bindings available
- Simple, focused API
- Active maintenance

**Cons**:
- No documented WASM support
- No `no_std` support
- No serde (custom serialization needed)
- Distance via trait (need to implement)

**Distance Metrics**: Custom via `Point` trait (Euclidean example provided)

---

#### 4. hnswlib-rs (Feature-Rich)

**Repository**: https://github.com/jean-pierreBoth/hnswlib-rs

| Attribute | Value |
|-----------|-------|
| Stars | 222 |
| License | Apache-2.0 / MIT |
| Pure Rust | Yes |
| Serde | **Yes** |
| Multithreaded | **Yes** |

**Pros**:
- Most distance metrics (L1, L2, Cosine, Jaccard, Hamming, Hellinger, Jeffreys, Jensen-Shannon)
- Multithreaded insertion and search
- Serde support for persistence
- Memory-mapped data support
- Filtering support

**Cons**:
- No WASM documentation
- No `no_std` support
- Uses `parking_lot` (may have WASM issues)

**Distance Metrics**: L1, L2, Cosine, Jaccard, Hamming, Levenshtein, Hellinger, Jeffreys, Jensen-Shannon

---

#### 5. granne (Not Recommended)

**Repository**: https://github.com/granne/granne

| Attribute | Value |
|-----------|-------|
| Stars | 320 |
| License | MIT |
| Last Commit | **June 2021** (abandoned) |

**Not Recommended**: Last updated 3.5 years ago. Single distance metric (Cosine). No active maintenance.

---

## Libraries Excluded (Not Pure Rust)

| Library | Stars | Reason for Exclusion |
|---------|-------|---------------------|
| usearch | 3.5k | C++ core with Rust bindings via FFI |
| annoy | 14.1k | C++ with read-only Rust bindings |

These cannot be used for WASM without compiling C++ to WASM separately.

---

## Recommendation

### Primary: hora

**Best choice for WASM compilation**:
- Official WASM support with working JavaScript bindings
- Large community (2.7k stars)
- Multiple index types beyond HNSW
- No heavy dependencies

```toml
[dependencies]
hora = "0.1"
```

### Alternative: rust-cv/hnsw

**Best choice if no_std or minimal dependencies required**:
- `no_std` support enables true embedded/WASM compilation
- Serde support for index persistence
- MIT license
- Smaller, focused codebase

```toml
[dependencies]
hnsw = { version = "0.11", features = ["serde"] }
```

---

## Implementation Strategy

### Option A: hora (Simpler)

```rust
#[cfg(target_arch = "wasm32")]
use hora::core::ann_index::ANNIndex;

#[cfg(not(target_arch = "wasm32"))]
// Use vectorlite via SQLite extension
```

### Option B: rust-cv/hnsw (More Control)

```rust
#[cfg(target_arch = "wasm32")]
mod wasm_hnsw {
    use hnsw::{Hnsw, Params};
    use space::Euclidean;

    pub fn create_index(dim: usize) -> Hnsw<Euclidean, Vec<f32>, usize> {
        Hnsw::new(Params::default())
    }
}
```

---

## Feature Comparison with vectorlite.so

| Feature | vectorlite.so | hora | rust-cv/hnsw |
|---------|--------------|------|--------------|
| HNSW Algorithm | Yes | Yes | Yes |
| L2 Distance | Yes | Yes | Yes (via trait) |
| Cosine Distance | Yes | Yes | Yes (via trait) |
| SIMD Acceleration | Google Highway | Built-in | Via `space` crate |
| Persistence | Via SQLite | No | Serde |
| WASM Compatible | No | **Yes** | Likely (no_std) |
| SQLite Integration | Native | Manual | Manual |

---

## SQLite Integration via rusqlite Virtual Tables

**Yes, hora (or any pure-Rust HNSW) can be integrated with SQLite** using rusqlite's virtual table API.

### Current vectorlite SQL Interface

```sql
-- Create collection
CREATE VIRTUAL TABLE vt_users USING vectorlite(
    vector_embedding float32[128] cosine,
    hnsw(max_elements=100000)
);

-- Insert vector
INSERT INTO vt_users(rowid, vector_embedding)
VALUES (1, vector_from_json('[0.1, 0.2, ...]'));

-- Search
SELECT rowid, distance FROM vt_users
WHERE knn_search(vector_embedding, knn_param(vector_from_json(?), 10));
```

### Implementing the Same Interface with hora

Using [rusqlite's vtab module](https://docs.rs/rusqlite/latest/rusqlite/vtab/index.html):

```rust
use rusqlite::vtab::{self, VTab, VTabCursor, IndexInfo, Context};
use hora::core::ann_index::ANNIndex;
use hora::index::hnsw_idx::HNSWIndex;

// 1. Define the virtual table structure
#[repr(C)]
struct HoraVTab {
    base: vtab::sqlite3_vtab,
    index: HNSWIndex<f32, usize>,
    dimension: usize,
}

// 2. Define the cursor for iteration
#[repr(C)]
struct HoraVTabCursor<'vtab> {
    base: vtab::sqlite3_vtab_cursor,
    vtab: &'vtab HoraVTab,
    results: Vec<(usize, f32)>,  // (rowid, distance)
    current: usize,
}

// 3. Implement VTab trait
impl VTab<'_> for HoraVTab {
    type Aux = ();
    type Cursor = HoraVTabCursor<'_>;

    fn connect(
        db: &mut vtab::VTabConnection,
        _aux: Option<&()>,
        args: &[&[u8]],
    ) -> rusqlite::Result<(String, Self)> {
        // Parse: CREATE VIRTUAL TABLE x USING hora(dim=128, metric=cosine)
        let dimension = parse_dimension(args);
        let schema = format!(
            "CREATE TABLE x(rowid INTEGER, vector_embedding BLOB, distance REAL)"
        );

        let index = HNSWIndex::new(
            dimension,
            &hora::index::hnsw_params::HNSWParams::default(),
        );

        Ok((schema, HoraVTab {
            base: vtab::sqlite3_vtab::default(),
            index,
            dimension,
        }))
    }

    fn best_index(&self, info: &mut IndexInfo) -> rusqlite::Result<()> {
        // Handle knn_search constraint
        info.set_estimated_cost(1000.0);
        Ok(())
    }

    fn open(&mut self) -> rusqlite::Result<Self::Cursor> {
        Ok(HoraVTabCursor {
            base: vtab::sqlite3_vtab_cursor::default(),
            vtab: self,
            results: vec![],
            current: 0,
        })
    }
}

// 4. Implement VTabCursor trait
impl VTabCursor for HoraVTabCursor<'_> {
    fn filter(&mut self, _idx: i32, _name: Option<&str>, args: &vtab::Values<'_>)
        -> rusqlite::Result<()>
    {
        // Parse query vector and k from args
        let query_vec: Vec<f32> = parse_vector(args);
        let k: usize = parse_k(args);

        // Perform HNSW search
        self.results = self.vtab.index.search(&query_vec, k);
        self.current = 0;
        Ok(())
    }

    fn next(&mut self) -> rusqlite::Result<()> {
        self.current += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.current >= self.results.len()
    }

    fn column(&self, ctx: &mut Context, col: i32) -> rusqlite::Result<()> {
        let (rowid, distance) = &self.results[self.current];
        match col {
            0 => ctx.set_result(*rowid as i64)?,
            2 => ctx.set_result(*distance as f64)?,
            _ => ctx.set_result(rusqlite::types::Null)?,
        }
        Ok(())
    }

    fn rowid(&self) -> rusqlite::Result<i64> {
        Ok(self.results[self.current].0 as i64)
    }
}

// 5. Register with SQLite
fn register_hora_vtab(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let module = vtab::update_module::<HoraVTab>();
    conn.create_module("hora", module, None)?;

    // Also register helper functions
    conn.create_scalar_function("vector_from_json", 1, |ctx| {
        // Parse JSON to Vec<f32>
    })?;

    conn.create_scalar_function("knn_param", 2, |ctx| {
        // Bundle query vector + k
    })?;

    Ok(())
}
```

### Key Points

| Aspect | vectorlite.so | hora + rusqlite vtab |
|--------|--------------|---------------------|
| Loading | Dynamic `.so` | Static Rust code |
| WASM | No | Yes |
| SQL syntax | Same | Same (can match exactly) |
| Performance | C++ optimized | Pure Rust |
| Build | Precompiled binary | Compiled with app |

### Required Dependencies

```toml
[dependencies]
rusqlite = { version = "0.37", features = ["bundled", "vtab"] }
hora = "0.1"
```

### Unified Architecture (All Platforms)

```
┌─────────────────────────────────────────────────────────┐
│                    VectorXLite API                       │
│              (create_collection, insert, search)         │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              hora + rusqlite virtual table               │
│                  (pure Rust, static)                     │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│     Linux     │   │     macOS     │   │     WASM      │
│   (native)    │   │   (native)    │   │   (browser)   │
└───────────────┘   └───────────────┘   └───────────────┘
```

**Benefits of unified approach:**
- Single codebase for all platforms
- No native binary dependencies (.so/.dylib/.dll to manage)
- Easier testing and maintenance
- Consistent behavior across platforms
- Removes 8MB+ of embedded binaries from repo

---

## Next Steps

1. **Create hora virtual table module** (`embedded/core/src/vtab/hora_vtab.rs`)
   - Implement `VTab` trait wrapping hora's HNSW index
   - Implement `VTabCursor` trait for search iteration
   - Register `vector_from_json()` and `knn_param()` scalar functions

2. **Update Cargo.toml**
   ```toml
   [dependencies]
   rusqlite = { version = "0.37", features = ["bundled", "vtab"] }
   hora = "0.1"
   ```

3. **Replace extension loader** (`embedded/core/src/helper/extension_loader.rs`)
   - Remove `include_bytes!` for .so/.dylib/.dll
   - Replace with `conn.create_module("vectorlite", hora_module, None)`

4. **Remove binary assets**
   ```
   rm embedded/core/assets/vectorlite.so
   rm embedded/core/assets/vectorlite.dylib
   rm embedded/core/assets/vectorlite.dll
   ```

5. **Test on all platforms**
   - Linux native
   - macOS native
   - WASM (browser)

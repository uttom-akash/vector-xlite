<p align="center">
  <img src="https://i.imgur.com/S3PJvXm.png" alt="VectorXLite Logo" width="80"/>
</p>

<h1 align="center">VectorXLite - Embedded Mode</h1>

<p align="center">
  <strong>In-process vector database library for Rust applications</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/vector_xlite"><img src="https://img.shields.io/crates/v/vector_xlite.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/vector_xlite"><img src="https://docs.rs/vector_xlite/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/uttom-akash/vector-xlite/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

---

The embedded mode allows you to use VectorXLite as a direct dependency in your Rust application, providing high-performance vector search with zero network overhead.

## Overview

The embedded library combines SQLite's reliability with HNSW-based vector search, enabling:
- **Sub-millisecond similarity search** on millions of vectors
- **SQL-powered payload filtering** for hybrid queries
- **ACID transactions** with atomic operations
- **In-memory or file-backed** storage options

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vector_xlite = { path = "../embedded/core" }  # Or from crates.io: "1.2"
r2d2 = "0.8"
r2d2_sqlite = "0.31"
rusqlite = { version = "0.37", features = ["load_extension"] }
```

## Quick Start

```rust
use vector_xlite::{VectorXLite, customizer::SqliteConnectionCustomizer, types::*};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create connection pool
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(10)
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)?;

    let db = VectorXLite::new(pool)?;

    // 2. Create a collection
    let config = CollectionConfigBuilder::default()
        .collection_name("products")
        .vector_dimension(384)
        .distance(DistanceFunction::Cosine)
        .payload_table_schema(
            "CREATE TABLE products (
                rowid INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT,
                price REAL
            )"
        )
        .build()?;

    db.create_collection(config)?;

    // 2.5. Check if collection exists
    if db.collection_exists("products")? {
        println!("Collection 'products' already exists!");
    }

    // 3. Insert vectors
    let embedding = vec![0.1; 384];
    let point = InsertPoint::builder()
        .collection_name("products")
        .id(1)
        .vector(embedding)
        .payload_insert_query(
            "INSERT INTO products(rowid, name, category, price)
             VALUES (?1, 'Wireless Headphones', 'Electronics', 99.99)"
        )
        .build()?;

    db.insert(point)?;

    // 4. Search with filtering
    let query = vec![0.15; 384];
    let search = SearchPoint::builder()
        .collection_name("products")
        .vector(query)
        .top_k(10)
        .payload_search_query(
            "SELECT rowid, name, category, price
             FROM products
             WHERE category = 'Electronics' AND price < 150"
        )
        .build()?;

    let results = db.search(search)?;

    for result in results {
        println!("Found: {}", result["name"]);
    }

    Ok(())
}
```

## Core Concepts

### Collections

A collection combines a vector index with a payload table.

#### Creating Collections

```rust
let config = CollectionConfigBuilder::default()
    .collection_name("documents")
    .vector_dimension(768)           // Dimension of your embeddings
    .distance(DistanceFunction::Cosine)  // Similarity metric
    .max_elements(1_000_000)         // Capacity
    .payload_table_schema(           // SQL schema for metadata
        "CREATE TABLE documents (
            rowid INTEGER PRIMARY KEY,
            title TEXT,
            content TEXT,
            tags JSON,
            created_at TIMESTAMP
        )"
    )
    .index_file_path("/data/docs.idx")  // Optional: persist HNSW index
    .build()?;
```

#### Checking Collection Existence

Before creating or using a collection, you can check if it exists:

```rust
// Check if a collection exists
if db.collection_exists("documents")? {
    println!("Collection exists!");
} else {
    println!("Collection does not exist, creating...");
    db.create_collection(config)?;
}

// Prevent duplicate collection creation
let collection_name = "products";
if !db.collection_exists(collection_name)? {
    let config = CollectionConfigBuilder::default()
        .collection_name(collection_name)
        .vector_dimension(384)
        .build()?;

    db.create_collection(config)?;
    println!("Collection created successfully");
} else {
    println!("Collection already exists, skipping creation");
}
```

**Important Notes:**
- Collection names are case-sensitive
- Empty collection names return an error
- This check is atomic and thread-safe
- Useful for idempotent initialization code

### Distance Functions

| Function | Description | Use Case |
|----------|-------------|----------|
| `Cosine` | Cosine similarity (normalized) | Text embeddings, NLP models |
| `L2` | Euclidean distance | Image features, spatial data |
| `IP` | Inner product (dot product) | Pre-normalized vectors |

### Storage Modes

**In-Memory (Development/Testing)**
```rust
let manager = SqliteConnectionManager::memory();
```

**File-Backed (Production)**
```rust
let manager = SqliteConnectionManager::file("vectors.db");
let config = CollectionConfigBuilder::default()
    .collection_name("prod")
    .index_file_path("/data/prod.idx")  // Persistent HNSW index
    .build()?;
```

## Advanced Usage

### JSON Payloads

```rust
let config = CollectionConfigBuilder::default()
    .collection_name("products")
    .payload_table_schema(
        "CREATE TABLE products (
            rowid INTEGER PRIMARY KEY,
            metadata JSON
        )"
    )
    .build()?;

// Insert
let point = InsertPoint::builder()
    .collection_name("products")
    .id(1)
    .vector(embedding)
    .payload_insert_query(
        r#"INSERT INTO products(rowid, metadata)
           VALUES (?1, '{"tags": ["sale"], "stock": 100}')"#
    )
    .build()?;

// Query JSON fields
let search = SearchPoint::builder()
    .collection_name("products")
    .vector(query)
    .payload_search_query(
        "SELECT * FROM products
         WHERE json_extract(metadata, '$.stock') > 0"
    )
    .build()?;
```

### Atomic Operations

```rust
// Atomic batch insert using transactions
let manager = SqliteConnectionManager::file("vectors.db");
let pool = Pool::builder()
    .connection_customizer(SqliteConnectionCustomizer::new())
    .build(manager)?;

let db = VectorXLite::new(pool.clone())?;

// All inserts are atomic - either all succeed or all fail
for (id, vector) in vectors.iter().enumerate() {
    db.insert(InsertPoint::builder()
        .collection_name("batch")
        .id(id as u64)
        .vector(vector.clone())
        .payload_insert_query(&format!(
            "INSERT INTO batch(rowid, data) VALUES (?1, 'Item {}')", id
        ))
        .build()?
    )?;
}
```

### Connection Pooling

```rust
use vector_xlite::customizer::SqliteConnectionCustomizer;

// Default timeout: 15 seconds
let customizer = SqliteConnectionCustomizer::new();

// Custom timeout: 30 seconds
let customizer = SqliteConnectionCustomizer::with_busy_timeout(30000);

let pool = Pool::builder()
    .max_size(15)  // Max concurrent connections
    .connection_customizer(customizer)
    .build(manager)?;
```

## Examples

Run the included examples:

```bash
# Run all examples
cargo run -p embedded-examples --release

# Run specific example
cd embedded/examples/rust
cargo run --release
```

## Performance Tips

1. **Use file-backed storage** for datasets larger than RAM
2. **Tune `max_elements`** to your expected dataset size
3. **Index payload columns** that you frequently filter on:
   ```sql
   CREATE INDEX idx_category ON products(category);
   ```
4. **Batch operations** for better throughput
5. **Persistent HNSW index** with `index_file_path` for faster restarts

## API Reference

### Main Types

- **`VectorXLite`** - Main database interface
  - `create_collection(config)` - Create a new collection
  - `collection_exists(name)` - Check if a collection exists
  - `insert(point)` - Insert a vector with payload
  - `search(query)` - Search for similar vectors
- **`CollectionConfigBuilder`** - Configure collections
- **`InsertPoint`** - Insert operations
- **`SearchPoint`** - Search operations
- **`DistanceFunction`** - Similarity metrics (Cosine, L2, IP)

### Full documentation

See the [main README](../README.md) for comprehensive API documentation.

## Architecture

```
┌─────────────────────────────────────────┐
│          Your Application               │
├─────────────────────────────────────────┤
│         VectorXLite API                 │
├──────────────────┬──────────────────────┤
│   HNSW Index     │      SQLite          │
│ (Vector Search)  │  (Payload Storage)   │
├──────────────────┴──────────────────────┤
│         Connection Pool (r2d2)          │
└─────────────────────────────────────────┘
```

## Use Cases

- **RAG Applications** - Embed documents for LLM context retrieval
- **Semantic Search** - Find similar content by meaning
- **Recommendation Systems** - Similar item suggestions
- **Image Search** - Visual similarity using CNN embeddings
- **Anomaly Detection** - Outlier detection in high-dimensional data
- **Deduplication** - Find near-duplicate content

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test -p vector_xlite_tests --release

# Run specific test file
cargo test -p vector_xlite_tests --test snapshot_tests --release
```

## Next Steps

- **Standalone Mode**: Need multi-language clients? See [../standalone/](../standalone/)
- **Distributed Mode**: Need high availability? See [../distributed/](../distributed/)
- **Documentation**: Full API docs at [docs.rs/vector_xlite](https://docs.rs/vector_xlite)

## License

MIT OR Apache-2.0

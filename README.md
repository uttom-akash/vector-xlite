<p align="center">
  <img src="https://i.imgur.com/S3PJvXm.png" alt="VectorXLite Logo" width="80"/>
</p>

<h1 align="center">VectorXLite</h1>

<p align="center">
  <strong>A fast, lightweight vector database with SQL-powered payload filtering</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/vector_xlite"><img src="https://img.shields.io/crates/v/vector_xlite.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/vector_xlite"><img src="https://docs.rs/vector_xlite/badge.svg" alt="Documentation"></a>
  <a href="https://github.com/anthropics/vector-db-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

---

## Overview

**VectorXLite** is a high-performance, embeddable vector database built on SQLite. It combines the power of HNSW-based approximate nearest neighbor search with the flexibility of SQL for metadata filtering, making it ideal for AI/ML applications, semantic search, and recommendation systems.

### Why VectorXLite?

| Feature | Benefit |
|---------|---------|
| **Embedded Architecture** | No separate server required - runs in-process |
| **SQLite Foundation** | Battle-tested storage with ACID guarantees |
| **HNSW Index** | Sub-millisecond similarity search on millions of vectors |
| **SQL Filtering** | Full SQL support for complex payload queries |
| **Atomic Operations** | Transaction support for data consistency |
| **Zero Configuration** | Works out of the box with sensible defaults |

---

## Features

- **Multiple Distance Functions**: Cosine similarity, L2 (Euclidean), and Inner Product
- **Flexible Dimensions**: Support for vectors of any dimension
- **Rich Payload Support**: Store and query arbitrary metadata alongside vectors
- **Hybrid Search**: Combine vector similarity with SQL WHERE clauses
- **Connection Pooling**: Built-in r2d2 pool support for concurrent access
- **Persistent Storage**: File-backed or in-memory operation modes
- **Type-Safe API**: Builder pattern with compile-time validation

---

## Installation

Add VectorXLite to your `Cargo.toml`:

```toml
[dependencies]
vector_xlite = "0.1"
r2d2 = "0.8"
r2d2_sqlite = "0.24"
```

---

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
        .vector_dimension(384)  // e.g., sentence-transformers output
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

    // 3. Insert vectors with metadata
    let embedding = vec![0.1, 0.2, 0.3, /* ... 384 dimensions */];

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

    // 4. Search with payload filtering
    let query_vector = vec![0.15, 0.25, 0.35, /* ... */];

    let search = SearchPoint::builder()
        .collection_name("products")
        .vector(query_vector)
        .top_k(10)
        .payload_search_query(
            "SELECT rowid, name, category, price
             FROM products
             WHERE category = 'Electronics' AND price < 150"
        )
        .build()?;

    let results = db.search(search)?;

    for result in results {
        println!("Found: {} - ${}", result["name"], result["price"]);
    }

    Ok(())
}
```

---

## API Reference

### VectorXLite

The main entry point for all database operations.

```rust
// Create from connection pool
let db = VectorXLite::new(pool)?;

// Available operations
db.create_collection(config)?;  // Create a new collection
db.insert(point)?;              // Insert a vector with payload
db.search(search_point)?;       // Perform similarity search
```

### CollectionConfigBuilder

Configure a new vector collection.

| Method | Type | Description |
|--------|------|-------------|
| `collection_name` | `&str` | Unique identifier for the collection |
| `vector_dimension` | `u16` | Number of dimensions (default: 3) |
| `distance` | `DistanceFunction` | Similarity metric (default: Cosine) |
| `max_elements` | `usize` | Maximum vectors (default: 100,000) |
| `payload_table_schema` | `&str` | SQL CREATE TABLE statement |
| `index_file_path` | `&str` | Path for persistent HNSW index |

```rust
let config = CollectionConfigBuilder::default()
    .collection_name("embeddings")
    .vector_dimension(768)
    .distance(DistanceFunction::Cosine)
    .max_elements(1_000_000)
    .payload_table_schema("CREATE TABLE embeddings (rowid INTEGER PRIMARY KEY, data TEXT)")
    .index_file_path("/data/embeddings.idx")
    .build()?;
```

### InsertPoint

Insert vectors with associated metadata.

| Method | Type | Description |
|--------|------|-------------|
| `collection_name` | `&str` | Target collection |
| `id` | `u64` | Unique vector identifier |
| `vector` | `Vec<f32>` | The embedding vector |
| `payload_insert_query` | `&str` | SQL INSERT statement (use `?1` for rowid) |

```rust
let point = InsertPoint::builder()
    .collection_name("documents")
    .id(42)
    .vector(embedding)
    .payload_insert_query("INSERT INTO documents(rowid, title) VALUES (?1, 'My Doc')")
    .build()?;
```

### SearchPoint

Configure similarity search queries.

| Method | Type | Description |
|--------|------|-------------|
| `collection_name` | `&str` | Collection to search |
| `vector` | `Vec<f32>` | Query vector |
| `top_k` | `i32` | Number of results (default: 10) |
| `payload_search_query` | `&str` | SQL SELECT for payload filtering |

```rust
let search = SearchPoint::builder()
    .collection_name("documents")
    .vector(query_embedding)
    .top_k(20)
    .payload_search_query("SELECT * FROM documents WHERE status = 'active'")
    .build()?;
```

### Distance Functions

| Function | Description | Best For |
|----------|-------------|----------|
| `Cosine` | Cosine similarity (normalized) | Text embeddings, NLP |
| `L2` | Euclidean distance | Image features, spatial data |
| `IP` | Inner product (dot product) | When vectors are pre-normalized |

---

## Storage Modes

### In-Memory (Development/Testing)

```rust
let manager = SqliteConnectionManager::memory();
let pool = Pool::builder()
    .connection_customizer(SqliteConnectionCustomizer::new())
    .build(manager)?;
```

### File-Backed (Production)

```rust
let manager = SqliteConnectionManager::file("vectors.db");
let pool = Pool::builder()
    .connection_customizer(SqliteConnectionCustomizer::new())
    .build(manager)?;

// With persistent HNSW index
let config = CollectionConfigBuilder::default()
    .collection_name("production")
    .index_file_path("/data/production.idx")
    // ... other config
    .build()?;
```

---

## Advanced Usage

### Complex Payload Queries with JOINs

```rust
// Create related tables
let author_table = "CREATE TABLE authors (id INTEGER PRIMARY KEY, name TEXT)";
let book_table = "CREATE TABLE books (
    rowid INTEGER PRIMARY KEY,
    author_id INTEGER,
    title TEXT,
    FOREIGN KEY (author_id) REFERENCES authors(id)
)";

// Search with JOIN
let search = SearchPoint::builder()
    .collection_name("books")
    .vector(query)
    .top_k(10)
    .payload_search_query(
        "SELECT b.rowid, b.title, a.name as author
         FROM books b
         JOIN authors a ON a.id = b.author_id
         WHERE a.name LIKE '%Smith%'"
    )
    .build()?;
```

### JSON Payload Support

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

// Insert with JSON
let point = InsertPoint::builder()
    .collection_name("products")
    .id(1)
    .vector(embedding)
    .payload_insert_query(
        r#"INSERT INTO products(rowid, metadata)
           VALUES (?1, '{"tags": ["sale", "new"], "stock": 100}')"#
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

### Custom Connection Timeout

```rust
use vector_xlite::customizer::SqliteConnectionCustomizer;

// Default timeout: 15 seconds
let customizer = SqliteConnectionCustomizer::new();

// Custom timeout (in milliseconds)
let customizer = SqliteConnectionCustomizer::with_busy_timeout(30000);

let pool = Pool::builder()
    .connection_customizer(customizer)
    .build(manager)?;
```

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Insert | O(log n) | HNSW index update |
| Search | O(log n) | Approximate nearest neighbor |
| Payload Filter | O(m) | SQLite query on matched vectors |

### Optimization Tips

1. **Batch Inserts**: Group multiple inserts in a single transaction
2. **Index Payload Columns**: Create SQLite indexes on frequently filtered columns
3. **Tune `max_elements`**: Set appropriately for your dataset size
4. **Use File Storage**: For datasets larger than available RAM

---

## Transaction Safety

VectorXLite provides atomic operations for data consistency:

```rust
// Both vector and payload are inserted atomically
// If either fails, the entire operation is rolled back
db.insert(point)?;
```

**Guarantees:**
- No orphan vectors (vectors without payload)
- No orphan payloads (payload without vectors)
- Failed operations don't affect existing data

---

## Use Cases

| Application | Description |
|-------------|-------------|
| **Semantic Search** | Find documents by meaning, not just keywords |
| **Recommendation Systems** | Similar item suggestions based on embeddings |
| **Image Search** | Find visually similar images using CNN features |
| **RAG Applications** | Retrieval-Augmented Generation for LLMs |
| **Anomaly Detection** | Find outliers in high-dimensional data |
| **Deduplication** | Identify near-duplicate content |

---

## Examples

The repository includes example applications:

```bash
# Run the basic example
cargo run -p example

# Run tests
cargo test
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     VectorXLite API                     │
├─────────────────────────────────────────────────────────┤
│  CollectionConfig  │  InsertPoint  │  SearchPoint      │
├─────────────────────────────────────────────────────────┤
│                    Query Planner                        │
├──────────────────────┬──────────────────────────────────┤
│    HNSW Index        │         SQLite                   │
│  (Vector Search)     │    (Payload Storage)             │
├──────────────────────┴──────────────────────────────────┤
│                 Connection Pool (r2d2)                  │
└─────────────────────────────────────────────────────────┘
```

---

## Requirements

- **Rust**: 1.70 or later
- **SQLite**: 3.35 or later (with extension loading enabled)
- **Platforms**: Linux, macOS, Windows

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Links

- [Crates.io](https://crates.io/crates/vector_xlite)
- [Documentation](https://docs.rs/vector_xlite)
- [GitHub Repository](https://github.com/uttom-akash/vector-xlite)

---

<p align="center">
  <sub>Built with Rust and SQLite</sub>
</p>

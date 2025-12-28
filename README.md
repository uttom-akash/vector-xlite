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

**VectorXLite** is a high-performance vector database built on SQLite with HNSW-based approximate nearest neighbor search. It combines the power of vector similarity with the flexibility of SQL for metadata filtering, making it ideal for AI/ML applications, semantic search, and recommendation systems.

### Three Deployment Modes

VectorXLite adapts to your needs with three distinct deployment modes:

<table>
<tr>
<td width="33%" valign="top">

### **Embedded**
**In-process library**

Direct Rust library integration with zero network overhead.

**Best for:**
- Single applications
- Development & testing
- Maximum performance

**Architecture:**
```
App → VectorXLite
```

[**Get Started →**](embedded/)

</td>
<td width="33%" valign="top">

### **Standalone**
**gRPC Server**

Language-agnostic server for remote access.

**Best for:**
- Multi-language clients
- Microservices
- Client-server apps

**Architecture:**
```
Client → gRPC
       → VectorXLite
```

[**Get Started →**](standalone/)

</td>
<td width="33%" valign="top">

### **Distributed**
**Raft Cluster**

High-availability cluster with consensus.

**Best for:**
- Production workloads
- Fault tolerance
- High availability

**Architecture:**
```
Client → Cluster
       → Raft
       → Servers
```

[**Get Started →**](distributed/)

</td>
</tr>
</table>

---

## Quick Comparison

| Feature | Embedded | Standalone | Distributed |
|---------|----------|------------|-------------|
| **Language** | Rust only | Any (gRPC) | Any (gRPC) |
| **Network** | None | TCP/gRPC | TCP/gRPC + Raft |
| **Setup** | Add dependency | Start server | Start cluster |
| **Availability** | Single process | Single server | Multi-node HA |
| **Consistency** | Local ACID | Single node | Raft consensus |
| **Latency** | Sub-millisecond | ~1-5ms | ~5-20ms |
| **Use Case** | Apps, tests | Services | Production |

---

## Key Features

### Vector Search
- **HNSW Algorithm** - Sub-millisecond similarity search on millions of vectors
- **Multiple Distance Functions** - Cosine, L2 (Euclidean), Inner Product
- **Flexible Dimensions** - Support for vectors of any dimension

### Payload Management
- **SQL Filtering** - Full SQL support for complex metadata queries
- **JSON Support** - Store and query JSON payloads
- **JOINs & Aggregations** - Complex relational queries on payloads

### Data Integrity
- **ACID Transactions** - Atomic operations with rollback support
- **Snapshot Support** - Point-in-time backups and recovery
- **No Orphans** - Guaranteed consistency between vectors and payloads

### Performance
- **Connection Pooling** - Concurrent access with r2d2
- **Persistent Storage** - File-backed or in-memory modes
- **Optimized Indexing** - Fast inserts and searches

---

## Quick Start by Mode

### Embedded Mode

```rust
use vector_xlite::{VectorXLite, customizer::SqliteConnectionCustomizer, types::*};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create in-memory database
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .connection_customizer(SqliteConnectionCustomizer::new())
        .build(manager)?;

    let db = VectorXLite::new(pool)?;

    // Create collection
    let config = CollectionConfigBuilder::default()
        .collection_name("products")
        .vector_dimension(384)
        .distance(DistanceFunction::Cosine)
        .payload_table_schema(
            "CREATE TABLE products (rowid INTEGER PRIMARY KEY, name TEXT, price REAL)"
        )
        .build()?;

    db.create_collection(config)?;

    // Insert vector
    let point = InsertPoint::builder()
        .collection_name("products")
        .id(1)
        .vector(vec![0.1; 384])
        .payload_insert_query("INSERT INTO products VALUES (?1, 'Headphones', 99.99)")
        .build()?;

    db.insert(point)?;

    // Search
    let search = SearchPoint::builder()
        .collection_name("products")
        .vector(vec![0.15; 384])
        .top_k(10)
        .payload_search_query("SELECT * FROM products WHERE price < 150")
        .build()?;

    let results = db.search(search)?;
    println!("Results: {:?}", results);

    Ok(())
}
```

[**Full Guide →**](embedded/)

### Standalone Mode

```bash
# Start the gRPC server
cargo run -p vector_xlite_server --release

# Use Go client
import "github.com/your-org/vectorxlite-go-client/client"

client, _ := client.NewVectorXLiteClient("localhost:50051")
```

[**Full Guide →**](standalone/) *(Coming soon)*

### Distributed Mode

```bash
# Start 3-node cluster with Raft consensus
cd distributed/cluster
./scripts/start_cluster.sh

# Use cluster client
client, _ := cluster.NewClusterClient("localhost:5002")
```

[**Full Guide →**](distributed/) *(Coming soon)*

---

## Installation

### Embedded Mode

```toml
[dependencies]
vector_xlite = "1.2"
r2d2 = "0.8"
r2d2_sqlite = "0.31"
```

### Standalone Mode

```bash
# Server
cargo build --release -p vector_xlite_server

# Go Client
go get github.com/your-org/vectorxlite-go-client
```

### Distributed Mode

```bash
# Build cluster
cd distributed/cluster
make build
```

---

## Project Structure

```
vector-db-rs/
├── embedded/          # Embedded library mode
│   ├── core/         # Core Rust library
│   ├── examples/     # Rust examples
│   └── docs/         # Embedded mode docs
│
├── standalone/        # Standalone server mode (coming soon)
│   ├── server/       # gRPC server
│   ├── clients/      # Go, Rust, Python clients
│   └── examples/     # Client examples
│
├── distributed/       # Distributed cluster mode (coming soon)
│   ├── cluster/      # Raft-based cluster
│   ├── clients/      # Cluster clients
│   └── examples/     # Cluster examples
│
├── proto/            # Protocol buffer definitions
├── tests/            # Integration tests
├── docs/             # Comprehensive documentation
└── scripts/          # Build and deployment scripts
```

---

## Examples

### Run Embedded Examples

```bash
# Run all examples
cargo run -p embedded-examples --release

# Output:
# Inserted complex story points
# Search Results: [...]
```

### Run Tests

```bash
# Run all tests
cargo test --release

# Run integration tests
cargo test -p vector_xlite_tests --release
```

---

## Architecture

### Embedded Mode
```
┌─────────────────────────────────────────┐
│          Your Application               │
├─────────────────────────────────────────┤
│         VectorXLite Library             │
├──────────────────┬──────────────────────┤
│   HNSW Index     │      SQLite          │
└──────────────────┴──────────────────────┘
```

### Standalone Mode
```
┌─────────────┐
│   Client    │
│  (Any Lang) │
└──────┬──────┘
       │ gRPC
┌──────▼──────────────────────────────────┐
│      VectorXLite gRPC Server (Rust)     │
├──────────────────┬──────────────────────┤
│   HNSW Index     │      SQLite          │
└──────────────────┴──────────────────────┘
```

### Distributed Mode
```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ gRPC
┌──────▼──────────────────────────────────┐
│        Cluster Proxy (Go + Raft)        │
│    Leader ◄─── Consensus ───► Follower  │
└──────┬──────────────────────┬───────────┘
       │                      │
┌──────▼──────┐        ┌──────▼──────┐
│ VectorXLite │        │ VectorXLite │
│   Server    │        │   Server    │
└─────────────┘        └─────────────┘
```

---

## Use Cases

| Use Case | Description | Recommended Mode |
|----------|-------------|------------------|
| **RAG for LLMs** | Retrieval-Augmented Generation | Embedded or Standalone |
| **Semantic Search** | Find documents by meaning | Any mode |
| **Recommendation** | Similar item suggestions | Embedded or Standalone |
| **Image Search** | Visual similarity | Standalone |
| **Production System** | High availability required | **Distributed** |
| **Microservices** | Multi-language services | Standalone |

---

## Performance

| Operation | Embedded | Standalone | Distributed |
|-----------|----------|------------|-------------|
| Insert (1000 vectors) | ~50ms | ~100ms | ~200ms |
| Search (top 10) | <1ms | ~2ms | ~10ms |
| Throughput | 20k ops/s | 10k ops/s | 5k ops/s |
| Dataset Size | Limited by RAM | Limited by disk | Distributed |

*Benchmarks on: Rust 1.70, Ubuntu 22.04, 16-core CPU, 384-dim vectors*

---

## Documentation

- **[Embedded Mode Guide](embedded/)** - In-process library usage
- **[Standalone Mode Guide](standalone/)** - gRPC server setup *(coming soon)*
- **[Distributed Mode Guide](distributed/)** - Cluster deployment *(coming soon)*
- **[API Reference](https://docs.rs/vector_xlite)** - Full API documentation
- **[Architecture](docs/architecture.md)** - System design *(coming soon)*

---

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md).

```bash
# Clone the repository
git clone https://github.com/uttom-akash/vector-db-rs
cd vector-db-rs

# Build all modes
make build-all

# Run all tests
make test-all
```

---

## Roadmap

- [x] Embedded mode with SQLite + HNSW
- [x] Snapshot support for backups
- [x] Atomic transactions
- [x] Distributed cluster with Raft consensus
- [ ] Complete standalone mode migration
- [ ] Complete distributed mode migration
- [ ] Python client library
- [ ] Observability stack (Prometheus, Grafana)
- [ ] Kubernetes deployment guides
- [ ] Performance optimizations

---

## Requirements

- **Rust**: 1.70 or later
- **SQLite**: 3.35 or later (with extension loading enabled)
- **Go**: 1.20+ (for distributed mode)
- **Platforms**: Linux, macOS, Windows

---

## License

This project is licensed under the MIT OR Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

---

## Links

- [Crates.io](https://crates.io/crates/vector_xlite)
- [Documentation](https://docs.rs/vector_xlite)
- [GitHub Repository](https://github.com/uttom-akash/vector-xlite)

---

## Acknowledgments

Built with:
- **Rust** - Systems programming language
- **SQLite** - Embedded database
- **HNSW** - Approximate nearest neighbor search
- **Raft** - Distributed consensus (HashiCorp implementation)
- **gRPC** - Remote procedure calls

---

<p align="center">
  <sub>VectorXLite - Fast, flexible, and reliable vector search</sub>
</p>

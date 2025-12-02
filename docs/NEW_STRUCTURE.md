# VectorXLite - Three-Mode Architecture Structure

This document defines a clear folder structure that reflects all three deployment modes: **Embedded**, **Standalone**, and **Distributed**.

## New Project Structure

```
vector-db-rs/
│
├── README.md                          # Main project README with mode selection guide
├── LICENSE
├── Cargo.toml                         # Rust workspace root
├── Makefile                           # Build automation for all modes
├── .gitignore
│
├── embedded/                          # MODE 1: Embedded Library
│   ├── README.md                      # "Using VectorXLite as Embedded Library"
│   ├── core/                          # Core Rust library (was: vector_xlite/)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── vector_xlite.rs
│   │   │   ├── executor/
│   │   │   ├── planner/
│   │   │   ├── types/
│   │   │   ├── error/
│   │   │   ├── snapshot/
│   │   │   ├── customizer/
│   │   │   ├── helper/
│   │   │   └── constant/
│   │   ├── assets/                    # SQLite extensions
│   │   │   ├── vectorlite.so
│   │   │   ├── vectorlite.dylib
│   │   │   └── vectorlite.dll
│   │   └── benches/                   # Performance benchmarks
│   │
│   ├── examples/                      # Embedded mode examples
│   │   ├── rust/
│   │   │   ├── basic-usage/
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/main.rs
│   │   │   ├── advanced-search/
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/main.rs
│   │   │   ├── json-payload/
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/main.rs
│   │   │   └── transactions/
│   │   │       ├── Cargo.toml
│   │   │       └── src/main.rs
│   │   │
│   │   └── python/                    # Future: Python bindings via PyO3
│   │       └── README.md
│   │
│   └── docs/
│       ├── getting-started.md
│       ├── api-reference.md
│       ├── performance-tuning.md
│       └── architecture.md
│
├── standalone/                        # MODE 2: Standalone gRPC Server
│   ├── README.md                      # "Running VectorXLite as Standalone Server"
│   ├── server/                        # Rust gRPC server (was: vector_xlite_grpc_server/)
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── Dockerfile
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── vector_xlite_grpc.rs
│   │   │   └── conversions.rs
│   │   └── config/
│   │       └── server.yaml
│   │
│   ├── clients/                       # Clients for VectorXLite gRPC API
│   │   ├── go/                        # Go client (was: vector_xlite_go_client/)
│   │   │   ├── README.md
│   │   │   ├── go.mod
│   │   │   ├── go.sum
│   │   │   ├── client/
│   │   │   │   ├── client.go
│   │   │   │   └── snapshot.go
│   │   │   ├── types/
│   │   │   │   ├── collection_config.go
│   │   │   │   ├── distance_func.go
│   │   │   │   ├── insert_point.go
│   │   │   │   ├── search_point.go
│   │   │   │   └── search_result.go
│   │   │   └── pb/                    # Generated from proto/vectorxlite/
│   │   │       ├── vectorxlite.pb.go
│   │   │       └── vectorxlite_grpc.pb.go
│   │   │
│   │   ├── rust/                      # Future: Rust client
│   │   │   └── README.md
│   │   │
│   │   └── python/                    # Future: Python client
│   │       └── README.md
│   │
│   ├── examples/                      # Standalone server examples
│   │   ├── go/
│   │   │   ├── basic-client/
│   │   │   │   ├── go.mod
│   │   │   │   └── main.go
│   │   │   ├── bulk-insert/
│   │   │   │   ├── go.mod
│   │   │   │   └── main.go
│   │   │   └── advanced-search/
│   │   │       ├── go.mod
│   │   │       └── main.go
│   │   │
│   │   └── rust/
│   │       └── README.md
│   │
│   ├── deployments/
│   │   ├── docker/
│   │   │   ├── Dockerfile
│   │   │   └── docker-compose.yml
│   │   └── kubernetes/
│   │       ├── deployment.yaml
│   │       ├── service.yaml
│   │       └── configmap.yaml
│   │
│   └── docs/
│       ├── getting-started.md
│       ├── api-reference.md
│       ├── deployment.md
│       └── performance.md
│
├── distributed/                       # MODE 3: Distributed Cluster with Raft
│   ├── README.md                      # "Running VectorXLite as Distributed Cluster"
│   ├── cluster/                       # Go-based cluster proxy (was: vector_xlite_proxy/)
│   │   ├── go.mod
│   │   ├── go.sum
│   │   ├── Makefile
│   │   │
│   │   ├── cmd/
│   │   │   ├── server/                # Main cluster node server (was: node/)
│   │   │   │   └── main.go
│   │   │   │
│   │   │   └── cli/                   # CLI management tool (was: client/)
│   │   │       └── main.go
│   │   │
│   │   ├── pkg/                       # Public/reusable packages
│   │   │   ├── consensus/             # Raft consensus implementation
│   │   │   │   ├── raft.go
│   │   │   │   ├── fsm.go             # Finite State Machine
│   │   │   │   ├── commands.go
│   │   │   │   └── snapshot.go
│   │   │   │
│   │   │   ├── server/                # Cluster gRPC server
│   │   │   │   ├── server.go
│   │   │   │   ├── interceptor.go
│   │   │   │   └── middleware.go
│   │   │   │
│   │   │   ├── client/                # Cluster client SDK
│   │   │   │   ├── client.go
│   │   │   │   └── interceptor.go
│   │   │   │
│   │   │   └── pb/                    # Generated cluster protos
│   │   │       ├── cluster.pb.go
│   │   │       └── cluster_grpc.pb.go
│   │   │
│   │   ├── internal/                  # Private packages
│   │   │   ├── config/
│   │   │   │   ├── config.go
│   │   │   │   └── loader.go
│   │   │   │
│   │   │   ├── observability/         # Observability implementation
│   │   │   │   ├── metrics/
│   │   │   │   │   ├── prometheus.go
│   │   │   │   │   ├── raft_metrics.go
│   │   │   │   │   ├── cluster_metrics.go
│   │   │   │   │   └── vectordb_metrics.go
│   │   │   │   │
│   │   │   │   ├── logging/
│   │   │   │   │   ├── logger.go
│   │   │   │   │   ├── context.go
│   │   │   │   │   └── interceptor.go
│   │   │   │   │
│   │   │   │   ├── tracing/
│   │   │   │   │   ├── tracer.go
│   │   │   │   │   ├── jaeger.go
│   │   │   │   │   └── interceptor.go
│   │   │   │   │
│   │   │   │   └── health/
│   │   │   │       ├── checker.go
│   │   │   │       └── probes.go
│   │   │   │
│   │   │   └── vectorxlite/           # VectorXLite client wrapper
│   │   │       └── client.go
│   │   │
│   │   ├── configs/                   # Node configurations
│   │   │   ├── node1.yaml
│   │   │   ├── node2.yaml
│   │   │   └── node3.yaml
│   │   │
│   │   ├── scripts/                   # Cluster management scripts
│   │   │   ├── start_cluster.sh
│   │   │   ├── stop_cluster.sh
│   │   │   └── test_operations.sh
│   │   │
│   │   ├── data/                      # Runtime data (gitignored)
│   │   │   ├── node1/
│   │   │   ├── node2/
│   │   │   └── node3/
│   │   │
│   │   ├── logs/                      # Application logs (gitignored)
│   │   └── bin/                       # Compiled binaries (gitignored)
│   │
│   ├── clients/                       # Clients for Cluster gRPC API
│   │   ├── go/
│   │   │   ├── README.md
│   │   │   ├── go.mod
│   │   │   ├── go.sum
│   │   │   ├── client/
│   │   │   │   ├── client.go
│   │   │   │   └── interceptor.go
│   │   │   └── pb/
│   │   │       ├── cluster.pb.go
│   │   │       └── cluster_grpc.pb.go
│   │   │
│   │   └── python/                    # Future: Python cluster client
│   │       └── README.md
│   │
│   ├── examples/                      # Distributed mode examples
│   │   └── go/
│   │       ├── cluster-setup/
│   │       │   ├── go.mod
│   │       │   └── main.go
│   │       ├── failover-demo/
│   │       │   ├── go.mod
│   │       │   └── main.go
│   │       └── distributed-ops/
│   │           ├── go.mod
│   │           └── main.go
│   │
│   ├── deployments/
│   │   ├── docker/
│   │   │   ├── Dockerfile
│   │   │   └── docker-compose.yml     # Full 3-node cluster
│   │   │
│   │   ├── kubernetes/
│   │   │   ├── statefulset.yaml
│   │   │   ├── service.yaml
│   │   │   ├── configmap.yaml
│   │   │   └── headless-service.yaml
│   │   │
│   │   └── observability/             # Observability stack
│   │       ├── prometheus/
│   │       │   ├── prometheus.yml
│   │       │   └── alerts.yml
│   │       ├── grafana/
│   │       │   ├── dashboards/
│   │       │   │   ├── cluster-overview.json
│   │       │   │   ├── raft-consensus.json
│   │       │   │   └── vectordb-operations.json
│   │       │   └── datasources.yml
│   │       ├── loki/
│   │       │   └── loki-config.yml
│   │       ├── jaeger/
│   │       │   └── jaeger-config.yml
│   │       └── docker-compose.yml     # Full observability stack
│   │
│   └── docs/
│       ├── getting-started.md
│       ├── raft-consensus.md
│       ├── topology.md
│       ├── observability.md
│       ├── operations.md
│       └── troubleshooting.md
│
├── proto/                             # Protocol buffer definitions (source of truth)
│   ├── vectorxlite/
│   │   ├── v1/
│   │   │   └── vectorxlite.proto
│   │   └── README.md
│   │
│   └── cluster/
│       ├── v1/
│       │   └── cluster.proto
│       └── README.md
│
├── tests/                             # Integration & E2E tests
│   ├── integration/                   # (was: vector_xlite_tests/)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   │       ├── common/
│   │       ├── atomic_transaction_tests.rs
│   │       ├── concurrent_tests.rs
│   │       ├── distance_function_tests.rs
│   │       ├── edge_case_tests.rs
│   │       ├── error_handling_tests.rs
│   │       ├── file_storage_tests.rs
│   │       ├── snapshot_tests.rs
│   │       └── sql_helper_tests.rs
│   │
│   ├── e2e/
│   │   ├── embedded/                  # E2E tests for embedded mode
│   │   │   └── README.md
│   │   ├── standalone/                # E2E tests for standalone mode
│   │   │   └── README.md
│   │   └── distributed/               # E2E tests for distributed mode
│   │       └── README.md
│   │
│   └── benchmarks/                    # Performance benchmarks
│       ├── embedded/
│       ├── standalone/
│       └── distributed/
│
├── docs/                              # Comprehensive documentation
│   ├── README.md                      # Documentation index
│   ├── overview.md                    # Project overview
│   ├── architecture.md                # Overall architecture
│   ├── choosing-mode.md               # Guide to choosing deployment mode
│   │
│   ├── embedded/
│   │   ├── README.md
│   │   ├── getting-started.md
│   │   ├── api-reference.md
│   │   └── best-practices.md
│   │
│   ├── standalone/
│   │   ├── README.md
│   │   ├── getting-started.md
│   │   ├── api-reference.md
│   │   ├── deployment.md
│   │   └── client-libraries.md
│   │
│   ├── distributed/
│   │   ├── README.md
│   │   ├── getting-started.md
│   │   ├── raft-consensus.md
│   │   ├── topology.md
│   │   ├── observability.md
│   │   ├── operations.md
│   │   └── troubleshooting.md
│   │
│   ├── guides/
│   │   ├── migration-guide.md         # Migrating between modes
│   │   ├── performance-tuning.md
│   │   ├── security.md
│   │   └── production-checklist.md
│   │
│   └── api/
│       ├── vectorxlite-api.md         # VectorXLite API
│       └── cluster-api.md             # Cluster API
│
├── scripts/                           # Project-wide scripts
│   ├── generate-protos.sh             # (was: protoc_gen.sh)
│   ├── build-all.sh
│   ├── test-all.sh
│   ├── run-embedded-example.sh
│   ├── run-standalone-example.sh
│   └── run-distributed-example.sh
│
├── assets/                            # Project assets
│   ├── logo.png
│   ├── transparent-logo.png
│   └── diagrams/
│       ├── embedded-architecture.svg
│       ├── standalone-architecture.svg
│       ├── distributed-architecture.svg
│       └── mode-comparison.svg
│
├── .github/
│   └── workflows/
│       ├── rust-ci.yml                # Embedded + Standalone tests
│       ├── go-ci.yml                  # Distributed tests
│       ├── integration-tests.yml      # Full E2E tests
│       └── release.yml
│
└── tools/                             # Development tools
    ├── migrate-structure.sh           # Migration script
    └── verify-structure.sh            # Verify new structure
```

## Three Deployment Modes

### **embedded/** - In-Process Library
- **Description**: Rust library embedded directly in applications
- **Use Case**: Single-process applications, development, testing
- **Components**: Core library, Rust examples, benchmarks
- **Network**: No network - direct function calls
- **Language**: Rust (with future Python bindings)

### **standalone/** - Single gRPC Server
- **Description**: Standalone gRPC server for remote access
- **Use Case**: Multi-language clients, microservices architecture
- **Components**: Rust server, Go/Rust/Python clients
- **Network**: Client ↔ gRPC Server
- **Language**: Server in Rust, clients in multiple languages

### **distributed/** - Clustered Raft System
- **Description**: Distributed cluster with Raft consensus
- **Use Case**: Production, high availability, fault tolerance
- **Components**: Go cluster proxy, Raft consensus, observability
- **Network**: Client ↔ Cluster Proxy (Raft) ↔ Standalone Server
- **Language**: Cluster in Go, leverages standalone server

## Migration Mapping

| Current Location | New Location | Mode |
|-----------------|--------------|------|
| `vector_xlite/` | `embedded/core/` | Embedded |
| `vector_xlite_grpc_server/` | `standalone/server/` | Standalone |
| `vector_xlite_go_client/` | `standalone/clients/go/` | Standalone |
| `vector_xlite_proxy/` | `distributed/cluster/` | Distributed |
| `vector_xlite_proxy/cmd/node/` | `distributed/cluster/cmd/server/` | Distributed |
| `vector_xlite_proxy/cmd/client/` | `distributed/cluster/cmd/cli/` | Distributed |
| `vector_xlite_tests/` | `tests/integration/` | Shared |
| `console_exmples/rust_examples/` | `embedded/examples/rust/` | Embedded |
| `console_exmples/go_examples/` | `standalone/examples/go/` | Standalone |
| `grpc_proto/` | `proto/` | Shared |
| `protoc_gen.sh` | `scripts/generate-protos.sh` | Shared |

## Root Cargo.toml (Rust Workspace)

```toml
[workspace]
members = [
    "embedded/core",
    "standalone/server",
    "tests/integration",
    "embedded/examples/rust/*",
]
resolver = "2"

[workspace.package]
version = "1.2.1"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/uttom-akash/vector-xlite"
authors = ["Uttom Akash <uttom.akash71@gmail.com>"]

[workspace.dependencies]
# Shared dependencies
rusqlite = { version = "0.37", features = ["load_extension", "backup"] }
r2d2 = "0.8"
r2d2_sqlite = "0.31"
regex = "1.12"
once_cell = "1.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
tonic = "0.11"
prost = "0.12"
```

## Root README.md Structure

```markdown
# VectorXLite - Multi-Mode Vector Database

[Logo and badges]

## Choose Your Deployment Mode

### 📦 Embedded Mode
**Best for:** Single applications, development, testing
- Direct in-process access
- Zero network overhead
- Simple integration
→ [Get Started](embedded/README.md)

### 🚀 Standalone Mode
**Best for:** Multi-language clients, microservices
- Language-agnostic gRPC access
- Remote operations
- Easy client integration
→ [Get Started](standalone/README.md)

### 🌐 Distributed Mode
**Best for:** Production, high availability, fault tolerance
- Raft consensus protocol
- Automatic failover
- Strong consistency guarantees
→ [Get Started](distributed/README.md)

## Architecture Overview

```
Embedded:      App → VectorXLite Library
Standalone:    Client → gRPC Server → VectorXLite Library
Distributed:   Client → Cluster (Raft) → gRPC Server → VectorXLite Library
```

[Rest of README]
```

## Benefits of This Structure

### ✅ **Clear Mode Separation**
```
embedded/     → In-process library usage
standalone/   → Single server deployment
distributed/  → Multi-node cluster
```

### ✅ **Intuitive Navigation**
```
Want embedded library? → embedded/
Want gRPC server? → standalone/
Want distributed cluster? → distributed/
Want examples? → [mode]/examples/
Want docs? → docs/[mode]/
```

### ✅ **Independent Evolution**
- Each mode can evolve independently
- Mode-specific optimizations
- Clear separation of concerns
- Easy to add features per mode

### ✅ **Shared Components**
- `proto/` - Single source of truth for APIs
- `tests/` - Comprehensive testing across modes
- `docs/` - Unified documentation
- `scripts/` - Build automation

## File Naming Convention

### Renamed Files/Directories
```bash
# Better naming for clarity
cmd/node/    → cmd/server/     # It's a server binary
cmd/client/  → cmd/cli/        # It's a CLI tool
fsm.go       → fsm.go          # Keep as-is (FSM is standard)
raft.go      → raft.go         # Keep as-is (clear enough)
```

## Next Steps

1. ✅ Review structure (current step)
2. Create migration script with git history preservation
3. Execute migration in phases
4. Update imports and paths
5. Update CI/CD pipelines
6. Update documentation
7. Test all three modes

Would you like me to proceed with creating the migration script?

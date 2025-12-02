# Recommended Project Structure for vector-db-rs

This document outlines the **Component-Based Architecture** - a manageable and scalable folder structure for the entire vector-db-rs project.

## Current Project Analysis

The project currently contains:
- **Core Rust library** (`vector_xlite`)
- **VectorXLite gRPC Server** (`vector_xlite_grpc_server`) - Serves vector operations
- **VectorXLite Go Client** (`vector_xlite_go_client`) - Client for VectorXLite gRPC server
- **Cluster Proxy with Raft** (`vector_xlite_proxy`) - Distributed layer with its own gRPC API
- **Cluster Client** - Client for Cluster gRPC API (inside proxy/pkg/client)
- **Integration tests** (`vector_xlite_tests`)
- **Examples** in both Rust and Go
- **Protocol buffers** for gRPC communication

## Key Insight: Two Separate Systems

This project has **two distinct components** that need clear separation:

1. **VectorXLite System**: Core vector database with its own gRPC API
   - VectorXLite Core Library
   - VectorXLite gRPC Server
   - VectorXLite Go Client

2. **Cluster System**: Distributed proxy layer with Raft consensus
   - Cluster Proxy Server (Raft-based)
   - Cluster Go Client (for cluster operations)
   - Cluster CLI tools

## Recommended Component-Based Structure

```
vector-db-rs/
│
├── vectorxlite/                   # Everything VectorXLite
│   ├── core/                     # Core Rust library
│   │   ├── src/
│   │   │   ├── constant/
│   │   │   ├── customizer/
│   │   │   ├── error/
│   │   │   ├── executor/
│   │   │   ├── helper/
│   │   │   ├── planner/
│   │   │   ├── snapshot/
│   │   │   ├── types/
│   │   │   ├── lib.rs
│   │   │   └── vector_xlite.rs
│   │   ├── assets/
│   │   │   ├── vectorlite.so
│   │   │   ├── vectorlite.dylib
│   │   │   └── vectorlite.dll
│   │   ├── benches/              # Performance benchmarks
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   ├── server/                   # VectorXLite gRPC server
│   │   ├── src/
│   │   │   ├── conversions.rs
│   │   │   ├── vector_xlite_grpc.rs
│   │   │   ├── lib.rs
│   │   │   └── main.rs
│   │   ├── build.rs
│   │   ├── Dockerfile
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   └── clients/
│       ├── go/                   # Go client for VectorXLite
│       │   ├── client/
│       │   │   ├── client.go
│       │   │   └── snapshot.go
│       │   ├── types/
│       │   │   ├── collection_config.go
│       │   │   ├── distance_func.go
│       │   │   ├── insert_point.go
│       │   │   ├── search_point.go
│       │   │   └── search_result.go
│       │   ├── pb/               # Generated from proto/vectorxlite/
│       │   │   ├── vectorxlite.pb.go
│       │   │   └── vectorxlite_grpc.pb.go
│       │   ├── go.mod
│       │   ├── go.sum
│       │   └── README.md
│       │
│       ├── rust/                 # Future: native Rust client
│       │   └── README.md
│       │
│       └── python/               # Future: Python client
│           └── README.md
│
├── cluster/                      # Everything Cluster/Distributed
│   ├── server/                   # Cluster proxy with Raft
│   │   ├── cmd/
│   │   │   ├── server/          # Main server (was: node)
│   │   │   │   └── main.go
│   │   │   └── cli/             # CLI tool (was: client)
│   │   │       └── main.go
│   │   │
│   │   ├── pkg/                 # Public/reusable packages
│   │   │   ├── consensus/       # Raft consensus layer
│   │   │   │   ├── state_machine.go  # (was: fsm.go)
│   │   │   │   ├── raft_node.go      # (was: raft.go)
│   │   │   │   ├── snapshot.go
│   │   │   │   └── commands.go
│   │   │   │
│   │   │   ├── client/          # Cluster client SDK/library
│   │   │   │   ├── client.go
│   │   │   │   └── interceptor.go
│   │   │   │
│   │   │   ├── server/          # Cluster server implementation
│   │   │   │   ├── server.go
│   │   │   │   ├── interceptor.go
│   │   │   │   └── middleware.go
│   │   │   │
│   │   │   └── pb/              # Generated cluster protos
│   │   │       ├── cluster.pb.go
│   │   │       └── cluster_grpc.pb.go
│   │   │
│   │   ├── internal/            # Private packages
│   │   │   ├── config/          # Configuration management
│   │   │   │   ├── config.go
│   │   │   │   └── loader.go
│   │   │   ├── metrics/         # Metrics & monitoring
│   │   │   │   ├── collector.go
│   │   │   │   └── prometheus.go
│   │   │   ├── logging/         # Structured logging
│   │   │   │   └── logger.go
│   │   │   └── health/          # Health checks
│   │   │       └── checker.go
│   │   │
│   │   ├── scripts/
│   │   │   ├── start_cluster.sh
│   │   │   ├── stop_cluster.sh
│   │   │   └── test_operations.sh
│   │   │
│   │   ├── configs/             # Node configurations
│   │   │   ├── node1.yaml
│   │   │   ├── node2.yaml
│   │   │   └── node3.yaml
│   │   │
│   │   ├── test/                # Integration & E2E tests
│   │   │   ├── integration/
│   │   │   └── e2e/
│   │   │
│   │   ├── data/                # Runtime data (gitignored)
│   │   │   ├── node1/
│   │   │   ├── node2/
│   │   │   └── node3/
│   │   │
│   │   ├── logs/                # Application logs
│   │   ├── bin/                 # Compiled binaries
│   │   ├── go.mod
│   │   ├── go.sum
│   │   ├── Makefile
│   │   └── README.md
│   │
│   └── clients/
│       ├── go/                   # Go client SDK (extracted from server/pkg/client)
│       │   ├── client/
│       │   │   ├── client.go
│       │   │   └── interceptor.go
│       │   ├── pb/              # Generated cluster protos
│       │   │   ├── cluster.pb.go
│       │   │   └── cluster_grpc.pb.go
│       │   ├── go.mod
│       │   ├── go.sum
│       │   └── README.md
│       │
│       └── python/               # Future: Python cluster client
│           └── README.md
│
├── proto/                        # Protocol buffer definitions (source of truth)
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
├── examples/                     # Usage examples
│   ├── vectorxlite/
│   │   ├── go/
│   │   │   ├── 01-basic-usage/
│   │   │   │   └── main.go
│   │   │   ├── 02-advanced-search/
│   │   │   │   └── main.go
│   │   │   └── 03-transactions/
│   │   │       └── main.go
│   │   │
│   │   └── rust/
│   │       ├── 01-basic-usage/
│   │       │   ├── src/
│   │       │   │   └── main.rs
│   │       │   └── Cargo.toml
│   │       └── 02-advanced-search/
│   │           ├── src/
│   │           │   └── main.rs
│   │           └── Cargo.toml
│   │
│   └── cluster/
│       └── go/
│           ├── 01-cluster-setup/
│           │   └── main.go
│           ├── 02-failover-demo/
│           │   └── main.go
│           └── 03-distributed-ops/
│               └── main.go
│
├── tests/
│   ├── integration/              # (was: vector_xlite_tests)
│   │   ├── tests/
│   │   │   ├── common/
│   │   │   ├── atomic_transaction_tests.rs
│   │   │   ├── concurrent_tests.rs
│   │   │   ├── distance_function_tests.rs
│   │   │   ├── edge_case_tests.rs
│   │   │   ├── error_handling_tests.rs
│   │   │   ├── file_storage_tests.rs
│   │   │   ├── snapshot_tests.rs
│   │   │   └── sql_helper_tests.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   │
│   └── e2e/
│       ├── vectorxlite/
│       │   └── README.md
│       └── cluster/
│           └── README.md
│
├── docs/
│   ├── vectorxlite/
│   │   ├── api.md
│   │   ├── architecture.md
│   │   └── getting-started.md
│   │
│   ├── cluster/
│   │   ├── api.md
│   │   ├── raft-consensus.md
│   │   └── topology.md
│   │
│   ├── guides/
│   │   ├── performance-tuning.md
│   │   └── troubleshooting.md
│   │
│   └── README.md
│
├── deployments/
│   ├── vectorxlite/
│   │   ├── docker/
│   │   │   ├── Dockerfile
│   │   │   └── docker-compose.yml
│   │   └── kubernetes/
│   │       ├── deployment.yaml
│   │       └── service.yaml
│   │
│   └── cluster/
│       ├── docker/
│       │   ├── Dockerfile
│       │   └── docker-compose.yml
│       └── kubernetes/
│           ├── statefulset.yaml
│           ├── service.yaml
│           └── configmap.yaml
│
├── scripts/
│   ├── generate-protos.sh        # (was: protoc_gen.sh at root)
│   ├── build-all.sh
│   ├── test-all.sh
│   └── README.md
│
├── assets/
│   ├── logo.png
│   ├── transparent-logo.png
│   └── diagrams/
│       ├── architecture.svg
│       └── cluster-topology.svg
│
├── .github/
│   └── workflows/
│       ├── rust-ci.yml
│       ├── go-ci.yml
│       └── release.yml
│
├── Cargo.toml                    # Rust workspace definition
├── Makefile                      # Common tasks automation
├── .gitignore
├── .editorconfig
├── LICENSE
└── README.md
```

## Key Design Principles

### 1. **Component-Based Organization**

Everything related to **VectorXLite** (core DB) lives under `vectorxlite/`:
- Core Rust library
- gRPC server
- All clients (Go, future Rust/Python)

Everything related to **Cluster** (distributed layer) lives under `cluster/`:
- Raft-based proxy server
- Cluster management clients
- Consensus logic

### 2. **Clear Distinction Between Two Systems**

| Component | Purpose | API | Client |
|-----------|---------|-----|--------|
| **VectorXLite** | Vector database operations | `vectorxlite.proto` | `vectorxlite/clients/go/` |
| **Cluster** | Distributed coordination | `cluster.proto` | `cluster/clients/go/` |

### 3. **Scalability**

- **Easy to add new clients**: Each component has a `clients/` directory
- **Easy to add new languages**: Go, Rust, Python, etc.
- **Versioned APIs**: Proto files organized in `v1/`, `v2/`, etc.
- **Independent deployment**: Each component can be deployed separately

### 4. **Developer Experience**

- **Intuitive navigation**: "Need VectorXLite client? Go to `vectorxlite/clients/go/`"
- **Clear examples**: Examples separated by component and language
- **Standard layouts**: Rust workspace, Go modules
- **Documentation**: Component-specific docs

## Migration Mapping

| Current Location | New Location | Type |
|-----------------|--------------|------|
| `vector_xlite/` | `vectorxlite/core/` | Rust library |
| `vector_xlite_grpc_server/` | `vectorxlite/server/` | Rust gRPC server |
| `vector_xlite_go_client/` | `vectorxlite/clients/go/` | Go client |
| `vector_xlite_proxy/` | `cluster/server/` | Go cluster server |
| `vector_xlite_proxy/pkg/client/` | `cluster/clients/go/` | Go cluster client |
| `vector_xlite_proxy/cmd/node/` | `cluster/server/cmd/server/` | Server binary |
| `vector_xlite_proxy/cmd/client/` | `cluster/server/cmd/cli/` | CLI tool |
| `vector_xlite_tests/` | `tests/integration/` | Integration tests |
| `console_exmples/` | `examples/` | Examples (fix typo) |
| `grpc_proto/` | `proto/` | Proto files |
| `protoc_gen.sh` | `scripts/generate-protos.sh` | Script |

## File Renaming Recommendations

### Within `cluster/server/pkg/consensus/`
```bash
fsm.go → state_machine.go           # More descriptive than FSM acronym
raft.go → raft_node.go              # Clearer what it contains
commands.go → raft_commands.go      # Indicates Raft-specific commands
```

### Within `cluster/server/cmd/`
```bash
node/ → server/                     # It's a server, not just a node
client/ → cli/                      # It's a CLI tool, not a client library
```

## Naming Conventions Summary

### **Servers**
- `vectorxlite/server/` - VectorXLite gRPC server (vector operations)
- `cluster/server/` - Cluster proxy server (Raft consensus, distributed ops)

### **Clients**
- `vectorxlite/clients/go/` - Go client for VectorXLite gRPC API
- `cluster/clients/go/` - Go client for Cluster gRPC API

### **Proto Files**
- `proto/vectorxlite/v1/vectorxlite.proto` - VectorXLite API
- `proto/cluster/v1/cluster.proto` - Cluster API

### **Examples**
- `examples/vectorxlite/go/` - How to use VectorXLite client
- `examples/cluster/go/` - How to use Cluster client

## Rust Workspace Configuration

**Root `Cargo.toml`:**

```toml
[workspace]
members = [
    "vectorxlite/core",
    "vectorxlite/server",
    "tests/integration",
    "examples/vectorxlite/rust/*",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/your-org/vector-db-rs"

[workspace.dependencies]
# Shared dependencies
rusqlite = "0.30"
r2d2 = "0.8"
r2d2_sqlite = "0.24"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
tonic = "0.11"
prost = "0.12"
```

## Go Module Organization

Each Go component maintains its own `go.mod`:

- **`vectorxlite/clients/go/go.mod`**: VectorXLite Go client module
- **`cluster/server/go.mod`**: Cluster server module
- **`cluster/clients/go/go.mod`**: Cluster Go client module (if extracted)
- **`examples/*/go.mod`**: Examples modules

Benefits:
- Independent versioning
- Separate dependency management
- Clean module boundaries
- Can be published separately to different repositories if needed

## Benefits of This Structure

### 1. **Clear Component Separation**
- VectorXLite and Cluster are distinct, easy to understand
- Each component has all related code together
- No confusion about which client/server you're working with

### 2. **Easy Navigation**
```
Need VectorXLite Go client? → vectorxlite/clients/go/
Need Cluster server? → cluster/server/
Need examples? → examples/vectorxlite/ or examples/cluster/
```

### 3. **Scalable**
- Add new client language? Just add to `clients/` directory
- Add new distributed component? Add to `cluster/` or create new top-level
- Each component can grow independently

### 4. **Professional**
- Follows industry standards
- Clear versioning strategy (proto v1, v2, etc.)
- Easy to extract components into separate repos later

### 5. **Developer Friendly**
- Obvious where everything lives
- Examples organized by component
- Documentation co-located with components

## Migration Strategy

### Phase 1: Create New Structure (No Breaking Changes)
1. Create `vectorxlite/`, `cluster/`, `proto/`, `examples/`, `docs/` directories
2. Copy (don't move) files to new locations
3. Verify everything works in new structure

### Phase 2: Update Build & Generation
1. Move proto files and update generation scripts
2. Update Rust workspace configuration
3. Update Go module paths
4. Regenerate proto code in new locations

### Phase 3: Migration
1. Use `git mv` to move files with history preservation
2. Update import paths in all files
3. Update CI/CD configurations
4. Update documentation

### Phase 4: Cleanup
1. Remove old directories
2. Update README and getting started guides
3. Tag a new version with the new structure

## Quick Reference: Two Systems

### VectorXLite System
```
vectorxlite/
├── core/              # Rust library
├── server/            # gRPC server
└── clients/
    ├── go/           # Go client → talks to vectorxlite/server
    ├── rust/         # Future
    └── python/       # Future
```
**Purpose**: Vector database operations (insert, search, collections)
**Proto**: `proto/vectorxlite/v1/vectorxlite.proto`

### Cluster System
```
cluster/
├── server/            # Raft-based proxy
│   ├── cmd/server/   # Server binary
│   └── cmd/cli/      # Management CLI
└── clients/
    ├── go/           # Go client → talks to cluster/server
    └── python/       # Future
```
**Purpose**: Distributed coordination, high availability, Raft consensus
**Proto**: `proto/cluster/v1/cluster.proto`

## Makefile Example

```makefile
.PHONY: help build test proto clean

help:
	@echo "Available targets:"
	@echo "  build          - Build all components"
	@echo "  test           - Run all tests"
	@echo "  proto          - Generate protobuf code"
	@echo "  clean          - Clean build artifacts"

build:
	@echo "Building VectorXLite..."
	cargo build --release --package vector_xlite_core
	cargo build --release --package vector_xlite_server
	@echo "Building Cluster..."
	cd cluster/server && go build -o bin/server cmd/server/main.go
	cd cluster/server && go build -o bin/cli cmd/cli/main.go
	@echo "Building clients..."
	cd vectorxlite/clients/go && go build ./...

test:
	@echo "Running Rust tests..."
	cargo test --workspace
	@echo "Running Go tests..."
	cd vectorxlite/clients/go && go test ./...
	cd cluster/server && go test ./...

proto:
	@echo "Generating protobuf code..."
	./scripts/generate-protos.sh

clean:
	cargo clean
	rm -rf cluster/server/bin
	rm -rf cluster/server/data/*
	rm -rf cluster/server/logs/*
```

## Proto Generation Script Example

**`scripts/generate-protos.sh`:**

```bash
#!/bin/bash
set -e

# Generate VectorXLite protos
echo "Generating VectorXLite protos..."
protoc --proto_path=proto/vectorxlite/v1 \
       --go_out=vectorxlite/clients/go/pb \
       --go-grpc_out=vectorxlite/clients/go/pb \
       proto/vectorxlite/v1/vectorxlite.proto

# Generate Cluster protos
echo "Generating Cluster protos..."
protoc --proto_path=proto/cluster/v1 \
       --go_out=cluster/server/pkg/pb \
       --go-grpc_out=cluster/server/pkg/pb \
       proto/cluster/v1/cluster.proto

# Optionally generate for Rust
echo "Generating Rust protos..."
# ... tonic build commands

echo "Proto generation complete!"
```

## Benefits Summary

| Aspect | Before | After | Benefit |
|--------|--------|-------|---------|
| **Component Clarity** | Mixed naming | Clear separation | Obvious what each part does |
| **Navigation** | Confusing paths | Intuitive structure | Find things quickly |
| **Client Distinction** | Unclear | Two separate clients | No confusion |
| **Scalability** | Hard to extend | Easy to add components | Future-proof |
| **Examples** | Mixed together | Organized by component | Easy to find relevant examples |
| **Documentation** | Scattered | Component-specific | Easier to maintain |
| **Versioning** | No versioning | Proto versioning (v1, v2) | API evolution support |
| **Independence** | Tightly coupled | Loosely coupled | Can deploy separately |

## Conclusion

This **component-based structure** provides:

✅ **Clear separation** between VectorXLite and Cluster systems
✅ **Easy navigation** - intuitive paths
✅ **Scalability** - easy to add new clients/languages
✅ **Professional** - industry-standard organization
✅ **Developer-friendly** - obvious where everything belongs
✅ **Future-proof** - supports growth and evolution

The structure recognizes that you have **two distinct systems**:
1. **VectorXLite**: Vector database with its own API
2. **Cluster**: Distributed coordination layer with its own API

Each system gets its own top-level directory with servers, clients, and documentation clearly organized.

## Observability for Distributed Clusters

Observability is **critical** for distributed systems, especially for a Raft-based cluster like vector_xlite_proxy. Proper instrumentation enables you to:
- Debug consensus issues across nodes
- Monitor cluster health and performance in real-time
- Detect leader elections and replication lag
- Track vector operation performance
- Ensure high availability and SLAs

### Recommended Observability Stack

For **vector_xlite_proxy**, we recommend the **industry-standard open-source stack** that maximizes job market relevance in Europe:

```
Prometheus (Metrics) + Grafana (Dashboards) + Loki (Logging) + Jaeger (Tracing) + OpenTelemetry (Standards)
```

**Why this stack?**
- ✅ **94% Grafana adoption** and **86% Prometheus usage** in production
- ✅ Most common setup for Kubernetes/cloud-native environments
- ✅ What European employers expect candidates to know
- ✅ Open-source and cost-effective
- ✅ Strong community support and documentation
- ✅ Native integration with each other

---

### The Three Pillars of Observability

#### 1. **Metrics** - Prometheus

Prometheus provides time-series metrics for quantitative system measurements.

**Key Metrics to Track:**

**Raft Consensus Metrics:**
```
raft_term_number                          # Current Raft term
raft_commit_index                         # Last committed log index
raft_applied_index                        # Last applied log index
raft_leader_changes_total                 # Number of leader elections
raft_heartbeat_duration_seconds           # Heartbeat latency (histogram)
raft_append_entries_total{status}         # AppendEntries RPC count
raft_log_replication_lag{follower_id}     # Replication lag per follower
raft_snapshot_create_duration_seconds     # Snapshot creation time
```

**VectorDB Operation Metrics:**
```
vectordb_operations_total{operation, status}       # insert/search/delete counts
vectordb_operation_duration_seconds{operation}     # p50, p95, p99 latencies
vectordb_collection_size{collection}               # Vectors per collection
vectordb_query_vector_count                        # Vectors scanned per query
vectordb_batch_size{operation}                     # Batch operation sizes
```

**Cluster Health Metrics:**
```
cluster_node_status{node_id, role}        # 1=leader, 0=follower
cluster_nodes_total                       # Total nodes in cluster
cluster_healthy_nodes                     # Healthy nodes count
cluster_request_failures_total            # Failed requests
cluster_request_retries_total             # Retry count
cluster_leader_redirects_total            # Redirects from followers to leader
```

**gRPC Metrics:**
```
grpc_server_handled_total{method, code}              # Request counts
grpc_server_handling_seconds{method}                 # Request duration
grpc_server_msg_received_total{method}               # Messages received
grpc_server_msg_sent_total{method}                   # Messages sent
```

**System Metrics:**
```
process_cpu_seconds_total                 # CPU usage
process_resident_memory_bytes             # Memory usage
go_goroutines                             # Active goroutines
go_gc_duration_seconds                    # GC pause times
```

#### 2. **Logging** - Grafana Loki

Loki provides log aggregation and querying, designed to work seamlessly with Grafana.

**Log Levels:**
- `ERROR` - System errors requiring immediate attention
- `WARN` - Warning conditions (degraded performance, retries)
- `INFO` - Important state changes (leader election, node join/leave)
- `DEBUG` - Detailed debugging information

**Important Events to Log:**

**Raft Events:**
```go
logger.Info("leader elected",
    "term", term,
    "leader_id", leaderID,
    "election_duration_ms", duration.Milliseconds(),
    "trace_id", traceID,
)

logger.Warn("log replication lag detected",
    "follower_id", followerID,
    "lag_entries", lag,
    "current_index", index,
)

logger.Info("snapshot created",
    "snapshot_id", snapshotID,
    "last_included_index", index,
    "size_bytes", size,
)
```

**Cluster Events:**
```go
logger.Info("node joined cluster",
    "node_id", nodeID,
    "address", addr,
    "cluster_size", size,
)

logger.Error("health check failed",
    "node_id", nodeID,
    "check_type", checkType,
    "error", err,
)
```

**VectorDB Events:**
```go
logger.Info("collection created",
    "collection", name,
    "dimension", dim,
    "distance_func", distFunc,
)

logger.Error("vector operation failed",
    "operation", op,
    "collection", collection,
    "error", err,
    "retry_count", retryCount,
)
```

**Structured Logging with Context:**
```go
// Use zerolog or zap for structured logging
log.Info().
    Str("trace_id", traceID).
    Str("node_id", nodeID).
    Str("operation", "insert_vectors").
    Int("batch_size", len(vectors)).
    Dur("duration_ms", duration).
    Msg("batch insert completed")
```

#### 3. **Tracing** - Jaeger + OpenTelemetry

Distributed tracing tracks requests across cluster nodes using OpenTelemetry standards.

**Why Tracing Matters for Raft Clusters:**
- Understand request flow: client → follower → leader → followers → client
- Identify bottlenecks in consensus protocol
- Debug complex distributed transactions
- Measure end-to-end latency across nodes

**Key Spans to Trace:**

**Client Request Path:**
```
client_insert_request (root span)
├── grpc_server_receive
├── request_validation
├── check_if_leader
│   └── redirect_to_leader (if follower)
├── raft_propose_command
│   ├── serialize_command
│   ├── append_to_local_log
│   ├── replicate_to_followers (parallel spans)
│   │   ├── replicate_to_node1
│   │   ├── replicate_to_node2
│   │   └── replicate_to_node3
│   └── wait_for_majority_commit
├── apply_to_state_machine
│   └── vectordb_insert_operation
│       ├── vectorxlite_grpc_call
│       ├── sql_prepare
│       ├── sql_execute
│       └── sql_commit
└── send_response_to_client
```

**OpenTelemetry Integration:**
```go
import (
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/trace"
)

// Start a span
ctx, span := otel.Tracer("cluster").Start(ctx, "raft_propose")
defer span.End()

// Add attributes
span.SetAttributes(
    attribute.String("node_id", nodeID),
    attribute.Int64("term", term),
    attribute.String("operation", "insert"),
)

// Propagate context across nodes via gRPC metadata
```

---

### Project Structure for Observability

```
cluster/server/
├── internal/
│   ├── observability/
│   │   ├── metrics/
│   │   │   ├── prometheus.go          # Prometheus registry & collectors
│   │   │   ├── raft_metrics.go        # Raft-specific metrics
│   │   │   ├── cluster_metrics.go     # Cluster health metrics
│   │   │   ├── vectordb_metrics.go    # Vector operation metrics
│   │   │   └── grpc_metrics.go        # gRPC interceptors with metrics
│   │   │
│   │   ├── logging/
│   │   │   ├── logger.go              # Logger initialization (zerolog/zap)
│   │   │   ├── context.go             # Request context propagation
│   │   │   ├── interceptor.go         # gRPC logging interceptor
│   │   │   └── formatters.go          # Log formatters for Loki
│   │   │
│   │   ├── tracing/
│   │   │   ├── tracer.go              # OpenTelemetry setup
│   │   │   ├── jaeger.go              # Jaeger exporter config
│   │   │   ├── interceptor.go         # gRPC tracing interceptor
│   │   │   └── span_helpers.go        # Span creation utilities
│   │   │
│   │   └── health/
│   │       ├── checker.go             # Health check implementation
│   │       ├── probes.go              # Liveness/readiness endpoints
│   │       └── dependencies.go        # Dependency health checks
│   │
│   └── config/
│       └── observability.go           # Observability config structs
│
├── deployments/
│   └── observability/
│       ├── prometheus/
│       │   ├── prometheus.yml         # Scrape configs
│       │   └── alerts.yml             # Alert rules
│       │
│       ├── grafana/
│       │   ├── dashboards/
│       │   │   ├── cluster-overview.json
│       │   │   ├── raft-consensus.json
│       │   │   ├── vectordb-operations.json
│       │   │   └── node-health.json
│       │   ├── datasources.yml        # Prometheus, Loki, Jaeger
│       │   └── grafana.ini
│       │
│       ├── loki/
│       │   └── loki-config.yml        # Log aggregation config
│       │
│       ├── jaeger/
│       │   └── jaeger-config.yml      # Tracing backend config
│       │
│       └── docker-compose.yml         # Full observability stack
│
├── configs/
│   └── node1.yaml
│       observability:
│         metrics:
│           enabled: true
│           port: 9090
│         logging:
│           level: info
│           format: json
│         tracing:
│           enabled: true
│           sample_rate: 0.1
```

---

### Docker Compose for Observability Stack

**`deployments/observability/docker-compose.yml`:**

```yaml
version: '3.8'

services:
  # Prometheus - Metrics collection
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/alerts.yml:/etc/prometheus/alerts.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.enable-lifecycle'
    networks:
      - observability

  # Grafana - Dashboards and visualization
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - ./grafana/datasources.yml:/etc/grafana/provisioning/datasources/datasources.yml
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - grafana-data:/var/lib/grafana
    depends_on:
      - prometheus
      - loki
      - jaeger
    networks:
      - observability

  # Loki - Log aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki/loki-config.yml:/etc/loki/local-config.yaml
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    networks:
      - observability

  # Promtail - Log shipper for Loki
  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./promtail/promtail-config.yml:/etc/promtail/config.yml
      - /var/log:/var/log
      - ../../../logs:/logs  # Cluster logs
    command: -config.file=/etc/promtail/config.yml
    depends_on:
      - loki
    networks:
      - observability

  # Jaeger - Distributed tracing
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "5775:5775/udp"   # Accept zipkin.thrift over compact thrift protocol
      - "6831:6831/udp"   # Accept jaeger.thrift over compact thrift protocol
      - "6832:6832/udp"   # Accept jaeger.thrift over binary thrift protocol
      - "5778:5778"       # Serve configs
      - "16686:16686"     # Serve frontend
      - "14268:14268"     # Accept jaeger.thrift directly from clients
      - "14250:14250"     # Accept model.proto (gRPC)
      - "9411:9411"       # Zipkin compatible endpoint
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    networks:
      - observability

  # Node Exporter - System metrics
  node-exporter:
    image: prom/node-exporter:latest
    ports:
      - "9100:9100"
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    networks:
      - observability

volumes:
  prometheus-data:
  grafana-data:
  loki-data:

networks:
  observability:
    driver: bridge
```

---

### Prometheus Configuration

**`deployments/observability/prometheus/prometheus.yml`:**

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Load alert rules
rule_files:
  - 'alerts.yml'

scrape_configs:
  # Cluster nodes
  - job_name: 'cluster-node1'
    static_configs:
      - targets: ['host.docker.internal:9091']
        labels:
          node_id: 'node1'
          cluster: 'vector-cluster'

  - job_name: 'cluster-node2'
    static_configs:
      - targets: ['host.docker.internal:9092']
        labels:
          node_id: 'node2'
          cluster: 'vector-cluster'

  - job_name: 'cluster-node3'
    static_configs:
      - targets: ['host.docker.internal:9093']
        labels:
          node_id: 'node3'
          cluster: 'vector-cluster'

  # System metrics
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

**`deployments/observability/prometheus/alerts.yml`:**

```yaml
groups:
  - name: raft_consensus
    rules:
      - alert: FrequentLeaderChanges
        expr: rate(raft_leader_changes_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Raft cluster unstable - frequent leader changes"
          description: "Leader has changed {{ $value }} times in the last 5 minutes"

      - alert: HighReplicationLag
        expr: raft_log_replication_lag > 1000
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "High replication lag on {{ $labels.follower_id }}"
          description: "Follower {{ $labels.follower_id }} is {{ $value }} entries behind"

      - alert: RaftNodeDown
        expr: up{job=~"cluster-.*"} == 0
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "Raft node {{ $labels.node_id }} is down"

  - name: cluster_health
    rules:
      - alert: ClusterUnhealthy
        expr: cluster_healthy_nodes < cluster_nodes_total
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "Cluster has unhealthy nodes"
          description: "Only {{ $value }} out of {{ $labels.cluster_nodes_total }} nodes are healthy"

      - alert: HighErrorRate
        expr: rate(cluster_request_failures_total[1m]) > 0.05
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High request error rate"
          description: "Error rate is {{ $value | humanizePercentage }}"

      - alert: HighLatency
        expr: histogram_quantile(0.99, rate(grpc_server_handling_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High p99 latency on {{ $labels.grpc_method }}"
          description: "p99 latency is {{ $value }}s"

  - name: vectordb_operations
    rules:
      - alert: VectorDBOperationFailures
        expr: rate(vectordb_operations_total{status="error"}[5m]) > 0.01
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High VectorDB operation failure rate"
          description: "{{ $labels.operation }} operations failing at {{ $value | humanizePercentage }}"
```

---

### Grafana Dashboards

**Key Dashboard Panels:**

**1. Cluster Overview Dashboard:**
- Current leader indicator
- Node health status (up/down)
- Request throughput (requests/sec)
- Error rate percentage
- P95/P99 latency graphs
- Active connections

**2. Raft Consensus Dashboard:**
- Current term timeline
- Leader election events (annotations)
- Log replication lag per follower (heatmap)
- AppendEntries success rate
- Commit index vs applied index gap
- Snapshot operations timeline

**3. VectorDB Operations Dashboard:**
- Operation breakdown pie chart (insert/search/delete)
- Query latency heatmap
- Operations per second by type
- Batch size distribution
- Collection sizes (top 10)
- Error rate by operation type

**4. Node Health Dashboard:**
- CPU usage per node
- Memory usage per node
- Goroutine count
- GC pause times
- Disk I/O
- Network throughput

---

### Health Checks

**Liveness Probe** (Is the process alive?):
```go
func (h *HealthChecker) Liveness(w http.ResponseWriter, r *http.Request) {
    // Basic checks
    if h.isShuttingDown() {
        w.WriteHeader(http.StatusServiceUnavailable)
        return
    }

    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(map[string]string{"status": "alive"})
}
```

**Readiness Probe** (Can it serve traffic?):
```go
func (h *HealthChecker) Readiness(w http.ResponseWriter, r *http.Request) {
    checks := []Check{
        h.checkRaftInitialized(),
        h.checkVectorXLiteConnection(),
        h.checkDiskSpace(),
    }

    allHealthy := true
    for _, check := range checks {
        if !check.Healthy {
            allHealthy = false
            break
        }
    }

    if allHealthy {
        w.WriteHeader(http.StatusOK)
    } else {
        w.WriteHeader(http.StatusServiceUnavailable)
    }

    json.NewEncoder(w).Encode(checks)
}
```

---

### Configuration Example

**`configs/node1.yaml`:**

```yaml
node:
  id: "node1"
  address: "localhost:8081"

observability:
  # Prometheus metrics
  metrics:
    enabled: true
    port: 9091
    path: "/metrics"

  # Structured logging
  logging:
    level: "info"          # debug, info, warn, error
    format: "json"         # json or console
    outputs:
      - "stdout"
      - "/var/log/cluster/node1.log"
    loki:
      enabled: true
      url: "http://localhost:3100/loki/api/v1/push"

  # Distributed tracing
  tracing:
    enabled: true
    exporter: "jaeger"
    jaeger:
      endpoint: "http://localhost:14268/api/traces"
    sample_rate: 0.1       # Trace 10% of requests (increase for debugging)

  # Health checks
  health:
    liveness_path: "/health/live"
    readiness_path: "/health/ready"
    port: 8081
```

---

### Implementation Roadmap

**Phase 1: Metrics Foundation (Week 1-2)**
1. ✅ Integrate Prometheus Go client
2. ✅ Create metrics registry and collectors
3. ✅ Instrument Raft consensus layer (term, commits, leader changes)
4. ✅ Add VectorDB operation metrics
5. ✅ Instrument gRPC server with metrics interceptors
6. ✅ Expose `/metrics` endpoint
7. ✅ Deploy Prometheus and configure scraping

**Phase 2: Dashboards (Week 2-3)**
1. ✅ Deploy Grafana
2. ✅ Create Prometheus datasource
3. ✅ Build Cluster Overview dashboard
4. ✅ Build Raft Consensus dashboard
5. ✅ Build VectorDB Operations dashboard
6. ✅ Build Node Health dashboard
7. ✅ Configure Prometheus alerts

**Phase 3: Logging (Week 3-4)**
1. ✅ Integrate structured logger (zerolog recommended)
2. ✅ Implement log levels and filtering
3. ✅ Add request correlation IDs (trace_id)
4. ✅ Create gRPC logging interceptor
5. ✅ Deploy Loki and Promtail
6. ✅ Configure Loki datasource in Grafana
7. ✅ Create log-based alerts

**Phase 4: Tracing (Week 4-5)**
1. ✅ Integrate OpenTelemetry SDK
2. ✅ Configure Jaeger exporter
3. ✅ Create gRPC tracing interceptors
4. ✅ Instrument Raft operations with spans
5. ✅ Propagate trace context across nodes
6. ✅ Deploy Jaeger backend
7. ✅ Configure Jaeger datasource in Grafana

**Phase 5: Health & Production (Week 5-6)**
1. ✅ Implement liveness/readiness probes
2. ✅ Add dependency health checks
3. ✅ Test alert firing and resolution
4. ✅ Create runbooks for common alerts
5. ✅ Load test with observability enabled
6. ✅ Optimize sampling rates and retention
7. ✅ Document observability setup

---

### Key Go Libraries

```go
// Prometheus metrics
"github.com/prometheus/client_golang/prometheus"
"github.com/prometheus/client_golang/prometheus/promauto"
"github.com/prometheus/client_golang/prometheus/promhttp"

// Structured logging (choose one)
"github.com/rs/zerolog/log"              // Recommended: Fast, zero-allocation
"go.uber.org/zap"                        // Alternative: Uber's logger

// OpenTelemetry tracing
"go.opentelemetry.io/otel"
"go.opentelemetry.io/otel/trace"
"go.opentelemetry.io/otel/exporters/jaeger"
"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
```

---

### Benefits Summary

✅ **Cost Effective** - Fully open-source stack, no licensing fees
✅ **Industry Standard** - 94% Grafana adoption, 86% Prometheus usage
✅ **Native Integration** - All tools designed to work together seamlessly
✅ **Cloud Native** - Perfect for Kubernetes and distributed systems
✅ **Active Community** - Extensive documentation and support
✅ **Scalable** - Handles millions of metrics, logs, and traces

---

## Next Steps

1. Review this proposal
2. Create migration script (preserving git history)
3. Execute migration in phases
4. Update CI/CD pipelines
5. Update documentation
6. Communicate changes to users
7. **Implement observability stack** (Prometheus + Grafana + Loki + Jaeger)
8. **Create dashboards and alerts**
9. **Load test with full observability**

---

**Last Updated**: 2025-12-02
**Status**: Proposed - Component-Based Architecture with Observability Stack
**Approved By**: User

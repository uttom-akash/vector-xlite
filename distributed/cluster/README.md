# VectorXLite Proxy - Distributed Vector Database Cluster

A distributed proxy layer for VectorXLite using Raft consensus for high availability and fault tolerance.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Node 1       │     │    Node 2       │     │    Node 3       │
│   (Leader)      │────▶│   (Follower)    │────▶│   (Follower)    │
│                 │     │                 │     │                 │
│ Raft: 5001      │     │ Raft: 5011      │     │ Raft: 5021      │
│ Cluster: 5002   │     │ Cluster: 5012   │     │ Cluster: 5022   │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │                       │                       │
   ┌─────▼──────┐          ┌─────▼──────┐          ┌─────▼──────┐
   │ VectorXLite│          │ VectorXLite│          │ VectorXLite│
   │  Server 1  │          │  Server 2  │          │  Server 3  │
   │ Port: 5003 │          │ Port: 5013 │          │ Port: 5023 │
   └────────────┘          └────────────┘          └────────────┘
```

## Port Convention

Each node uses a base port with specific suffixes:
- **xxx1**: Raft internal communication
- **xxx2**: Cluster gRPC API (client facing)
- **xxx3**: VectorXLite gRPC server (dedicated instance per node)

Example for base port 500:
- Node1: Raft=5001, Cluster=5002, VectorXLite=5003
- Node2: Raft=5011, Cluster=5012, VectorXLite=5013
- Node3: Raft=5021, Cluster=5022, VectorXLite=5023

## Project Structure

```
vector_xlite_proxy/
├── cmd/
│   ├── node/          # Node server CLI
│   └── client/        # Client CLI for operations
├── pkg/
│   ├── consensus/     # Raft consensus logic
│   ├── client/        # Client library
│   ├── server/        # Server implementation
│   └── pb/            # Protocol buffers
├── scripts/
│   ├── start_cluster.sh    # Start 3-node cluster
│   ├── stop_cluster.sh     # Stop cluster
│   └── test_operations.sh  # Test cluster operations
├── data/              # Raft data (gitignored)
├── logs/              # Application logs
└── bin/               # Compiled binaries
```

## Prerequisites

1. Go 1.20+ installed
2. Rust and Cargo installed (for VectorXLite server)
3. VectorXLite gRPC server directory at `/home/akash/Dev/vector-db-rs/vector_xlite_grpc_server`

## Quick Start

### 1. Start the Cluster

The start script will automatically start the VectorXLite server if it's not running:

```bash
./scripts/start_cluster.sh
```

This will:
- **Automatically start 3 VectorXLite gRPC servers** (ports 5003, 5013, 5023)
- Build the node and client binaries
- Start 3 proxy nodes (node1, node2, node3)
- Each node connects to its dedicated VectorXLite instance
- Join nodes to form a Raft cluster
- Display cluster information

### 2. Test Operations

```bash
./scripts/test_operations.sh
```

This will:
- Create a collection
- Insert vectors
- Search vectors
- Test read operations on followers
- Test write redirect from followers to leader

### 3. Stop the Cluster

```bash
# Stop only cluster nodes (keeps VectorXLite server running)
./scripts/stop_cluster.sh

# Stop cluster nodes AND VectorXLite server
./scripts/stop_cluster.sh --with-vector-server
```

## Manual Usage

### Start Individual Nodes

```bash
# Node 1 (bootstrap)
./bin/node -id node1 -port 500 -vector-addr "0.0.0.0:50051" -bootstrap

# Node 2
./bin/node -id node2 -port 501 -vector-addr "0.0.0.0:50051"

# Node 3
./bin/node -id node3 -port 502 -vector-addr "0.0.0.0:50051"
```

### Client Operations

#### Get Cluster Info
```bash
./bin/client info -addr :5002
```

#### Create Collection
```bash
./bin/client create-collection \
  -addr :5002 \
  -name users \
  -dim 128 \
  -schema "create table users(rowid integer primary key, name text)"
```

#### Insert Vector
```bash
./bin/client insert \
  -addr :5002 \
  -name users \
  -id 1 \
  -vector "1.0,2.0,3.0,4.0" \
  -query "insert into users(name) values ('Alice')"
```

#### Search Vectors
```bash
./bin/client search \
  -addr :5002 \
  -name users \
  -vector "1.0,2.0,3.0,4.0" \
  -k 5 \
  -query "select rowid, name from users"
```

#### Join Node to Cluster
```bash
./bin/client join \
  -addr :5002 \
  -node-id node2 \
  -node-addr 127.0.0.1:5011
```

## Features

### High Availability
- **Leader Election**: Automatic leader election using Raft consensus
- **Failover**: Automatic failover if leader fails
- **Replication**: All write operations replicated to followers

### Client-Side Operations
- **Write Redirect**: Writes automatically redirected to leader
- **Read from Any Node**: Reads can be served from any node
- **Automatic Retry**: Client handles leader redirection

### Consistency
- **Strong Consistency**: Raft consensus ensures strong consistency
- **Linearizable Writes**: All writes go through leader
- **Snapshot Support**: Periodic snapshots for faster recovery

## Development

### Build Binaries

```bash
# Build node
go build -o bin/node cmd/node/main.go

# Build client
go build -o bin/client cmd/client/main.go
```

### Run Tests

```bash
go test ./...
```

### View Logs

```bash
# Follow node logs
tail -f logs/node1.log
tail -f logs/node2.log
tail -f logs/node3.log
```

## Troubleshooting

### Cluster won't start
- Ensure VectorXLite server is running on port 50051
- Check if ports 5001, 5002, 5011, 5012, 5021, 5022 are available
- Check logs in `logs/` directory

### Write operations fail
- Ensure leader is elected: `./bin/client info -addr :5002`
- Wait a few seconds for leader election
- Check node logs for errors

### Nodes can't communicate
- Check firewall settings
- Verify raft addresses are reachable
- Check network connectivity between nodes

## License

MIT

## Contributing

Pull requests are welcome. For major changes, please open an issue first.

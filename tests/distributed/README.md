# Distributed Cluster Integration Tests

This directory contains integration tests for VectorXLite distributed cluster mode.

## Overview

These tests verify the complete functionality of the distributed cluster by:
- Testing cluster consensus and leader election
- Verifying automatic write redirection to the leader
- Testing read operations from any node
- Validating data replication across nodes
- Testing concurrent operations in a distributed environment
- Verifying collection management across the cluster

## Test Coverage

The integration tests cover:

1. **Cluster Management**
   - Retrieving cluster information from all nodes
   - Verifying leader election (exactly one leader)
   - Checking node states (leader/follower)

2. **Write Operations**
   - Creating collections (redirected to leader)
   - Inserting vectors (redirected to leader)
   - Verifying automatic redirection from followers to leader
   - Concurrent writes to the cluster

3. **Read Operations**
   - Searching from any node in the cluster
   - Verifying CollectionExists from any node
   - Testing read consistency after replication

4. **Data Replication**
   - Verifying data is replicated across all nodes
   - Testing eventual consistency
   - Validating searches return same results from all nodes

5. **Collection Operations**
   - Creating multiple collections in the cluster
   - Verifying collection isolation
   - Testing collection existence checks

## Prerequisites

- Go 1.22 or later
- A running 3-node VectorXLite cluster with the following configuration:
  - Node 1: localhost:5002 (cluster port)
  - Node 2: localhost:5012 (cluster port)
  - Node 3: localhost:5022 (cluster port)

## Setting Up the Cluster

Before running the tests, you need to start a 3-node cluster. Refer to the cluster documentation for setup instructions:

```bash
cd ../../distributed/cluster

# Start node 1 (bootstrap node)
# Start node 2
# Start node 3
```

Verify the cluster is healthy:
```bash
# Check if all nodes are listening
lsof -i :5002  # Node 1
lsof -i :5012  # Node 2
lsof -i :5022  # Node 3
```

## Running the Tests

### Using the Test Script (Recommended)

```bash
./run_tests.sh
```

This script will:
1. Check if cluster nodes are accessible
2. Verify at least one node is running
3. Run all integration tests
4. Report results

### Manual Testing

```bash
# Download dependencies
go mod download

# Run all tests
go test -v ./...

# Run specific test
go test -v -run TestClusterInfo

# Run with custom timeout
go test -v -timeout 5m ./...
```

## Test Configuration

The tests use these default addresses:
- Node 1: `localhost:5002`
- Node 2: `localhost:5012`
- Node 3: `localhost:5022`

To change these, modify the constants in `integration_test.go`:
```go
const (
    node1ClusterAddr = "localhost:5002"
    node2ClusterAddr = "localhost:5012"
    node3ClusterAddr = "localhost:5022"
)
```

## Individual Test Descriptions

### TestClusterInfo
Verifies that cluster information can be retrieved from all nodes and contains valid data.

### TestLeaderElection
Ensures exactly one node is elected as leader and all nodes agree on who the leader is.

### TestCollectionOperations
Tests the complete lifecycle of collections in a distributed environment:
- Create collection (auto-redirected to leader)
- Check collection exists
- Insert data (auto-redirected to leader)
- Search data

### TestWriteRedirection
Specifically tests that write operations to followers are automatically redirected to the leader.

### TestReadFromFollowers
Verifies that read operations (search, collection_exists) can be served by any node after replication.

### TestConcurrentWrites
Tests concurrent write operations from multiple goroutines to ensure cluster handles concurrency correctly.

### TestMultipleCollectionsInCluster
Validates that multiple collections can be created and managed independently in the cluster.

## Troubleshooting

### Cluster Not Running

```
Error: No cluster nodes are accessible
```

Solution: Start the cluster before running tests.

### Replication Lag

Some tests include `time.Sleep(2 * time.Second)` to allow for replication. If tests fail with "expected N results, got 0":
- Increase the sleep duration in the test
- Check cluster logs for replication errors
- Verify all nodes are healthy

### Leader Election Issues

If tests report multiple leaders or no leader:
- Check cluster logs for Raft errors
- Verify network connectivity between nodes
- Restart the cluster and retry

### Connection Timeouts

If tests timeout:
1. Increase test timeout: `go test -v -timeout 10m ./...`
2. Check firewall settings
3. Verify cluster ports are accessible: `telnet localhost 5002`

### Module Issues

```bash
go mod download
go mod tidy
```

## Writing New Tests

When adding new cluster tests:

1. Consider replication delay - add appropriate sleeps after writes
2. Test against multiple nodes when possible
3. Handle node failures gracefully (some nodes may be down)
4. Use unique collection names to avoid conflicts
5. Verify data consistency across nodes

Example test structure:
```go
func TestClusterFeature(t *testing.T) {
    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()

    // Connect to any node (writes auto-redirect to leader)
    c, err := client.NewClusterClientSimple(node1ClusterAddr)
    if err != nil {
        t.Fatalf("Failed to connect: %v", err)
    }
    defer c.Close()

    // Perform operations
    // ...

    // Wait for replication
    time.Sleep(2 * time.Second)

    // Verify across multiple nodes
    // ...
}
```

## Performance Considerations

Distributed tests are slower than standalone tests due to:
- Network latency between nodes
- Raft consensus protocol overhead
- Data replication delays

Typical test durations:
- Individual test: 5-30 seconds
- Full test suite: 2-5 minutes

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
- name: Setup Cluster
  run: |
    cd distributed/cluster
    # Start cluster nodes

- name: Run Distributed Integration Tests
  run: |
    cd tests/distributed
    ./run_tests.sh
```

## Cluster Health Monitoring

To monitor cluster health during tests:

```bash
# In another terminal
watch -n 1 'echo "=== Node 1 ===" && curl -s localhost:5002/health || echo "Down"'
```

## Related Documentation

- [Distributed Cluster Documentation](../../distributed/cluster/README.md)
- [Cluster Client Documentation](../../distributed/cluster/pkg/client/README.md)
- [Raft Consensus Documentation](../../distributed/cluster/pkg/consensus/README.md)

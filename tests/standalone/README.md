# Standalone Mode Integration Tests

This directory contains integration tests for VectorXLite standalone mode (gRPC server).

## Overview

These tests verify the complete functionality of the standalone gRPC server by:
- Starting a gRPC server instance
- Using the Go client to interact with the server
- Testing all API operations (create collection, insert, search, collection_exists, snapshots)
- Verifying concurrent operations
- Testing different distance functions and payload operations

## Test Coverage

The integration tests cover:

1. **Collection Lifecycle**
   - Creating collections
   - Checking collection existence
   - Managing multiple collections

2. **Vector Operations**
   - Inserting single vectors
   - Inserting multiple vectors
   - Searching with different distance functions (Cosine, L2, Inner Product)
   - Concurrent inserts and searches

3. **Payload Operations**
   - Creating collections with payload schemas
   - Inserting vectors with payload data
   - Searching with payload queries

4. **Snapshot Operations**
   - Exporting snapshots (sync and streaming)
   - Importing snapshots
   - Verifying data integrity after snapshot restore

5. **Edge Cases**
   - Empty collection names
   - Nonexistent collections
   - Concurrent operations

## Prerequisites

- Go 1.22 or later
- Rust toolchain (for building the server)
- The VectorXLite standalone server built and ready

## Running the Tests

### Option 1: Using the Test Script (Recommended)

The easiest way to run the tests is using the provided script:

```bash
./run_tests.sh
```

This script will:
1. Check if the server is already running
2. Build and start the server if needed
3. Run all integration tests
4. Stop the server when done (if it was started by the script)

### Option 2: Manual Testing

If you want to run the server manually:

1. Start the standalone server:
```bash
cd ../../standalone/server
cargo run --release
```

2. In another terminal, run the tests:
```bash
cd tests/standalone
go test -v ./...
```

## Test Configuration

The tests connect to the server at `localhost:50051` by default. If you need to change this, modify the `serverAddr` constant in `integration_test.go`.

## Individual Test Execution

To run specific tests:

```bash
# Run a specific test
go test -v -run TestCollectionLifecycle

# Run tests matching a pattern
go test -v -run TestCollection

# Run with timeout
go test -v -timeout 5m ./...
```

## Troubleshooting

### Server Connection Issues

If tests fail with connection errors:
1. Verify the server is running: `lsof -i :50051`
2. Check server logs for errors
3. Ensure no firewall is blocking port 50051

### Test Timeouts

If tests timeout:
1. Increase the timeout: `go test -v -timeout 10m ./...`
2. Check system resources (disk space, memory)
3. Verify the server is responding: `curl -v http://localhost:50051`

### Module Issues

If you get Go module errors:
```bash
go mod download
go mod tidy
```

## Writing New Tests

When adding new tests:

1. Follow the existing test naming convention: `Test<Feature><Scenario>`
2. Use the `client` package for all server interactions
3. Generate unique collection names using timestamps
4. Clean up resources when possible (though server restart will reset state)
5. Use appropriate context timeouts

Example test structure:
```go
func TestMyFeature(t *testing.T) {
    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()

    c, err := client.NewClient(ctx, serverAddr, dialTimeout)
    if err != nil {
        t.Fatalf("Failed to connect: %v", err)
    }
    defer c.Close()

    collectionName := fmt.Sprintf("test_%d", time.Now().UnixNano())
    // ... test logic ...
}
```

## CI/CD Integration

To integrate these tests into your CI/CD pipeline:

```yaml
# Example GitHub Actions
- name: Run Standalone Integration Tests
  run: |
    cd tests/standalone
    ./run_tests.sh
```

## Related Documentation

- [Standalone Server Documentation](../../standalone/server/README.md)
- [Go Client Documentation](../../standalone/clients/go/README.md)
- [VectorXLite Core Documentation](../../embedded/core/README.md)

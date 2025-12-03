# VectorXLite Integration Tests

This directory contains comprehensive integration tests for VectorXLite in various deployment modes.

## Overview

The integration tests are organized by deployment mode:

```
tests/
├── embedded/          # Tests for the embedded Rust library
├── standalone/        # Tests for standalone gRPC server mode
└── distributed/       # Tests for distributed cluster mode
```

## Test Suites

### 1. Embedded Tests (`embedded/`)

**Language:** Rust
**Target:** VectorXLite core library

These tests verify the core functionality of the VectorXLite embedded library:
- Collection creation and management
- Vector insertion and search
- Distance functions (Cosine, L2, Inner Product)
- Payload operations
- Snapshot functionality
- Concurrent operations
- Edge cases and error handling

**Run:**
```bash
cd embedded
cargo test
```

### 2. Standalone Tests (`standalone/`)

**Language:** Go
**Target:** Standalone gRPC server

These tests verify the standalone server deployment:
- gRPC API functionality
- Client-server communication
- Collection lifecycle management
- Vector operations via gRPC
- Snapshot export/import
- Concurrent client operations

**Run:**
```bash
cd standalone
./run_tests.sh
```

See [standalone/README.md](standalone/README.md) for details.

### 3. Distributed Tests (`distributed/`)

**Language:** Go
**Target:** Distributed cluster with Raft consensus

These tests verify the distributed cluster deployment:
- Leader election and consensus
- Automatic write redirection
- Read operations from any node
- Data replication across nodes
- Cluster management operations
- Concurrent distributed operations

**Run:**
```bash
cd distributed
./run_tests.sh
```

See [distributed/README.md](distributed/README.md) for details.

## Quick Start

### Prerequisites

- **For all tests:**
  - Git
  - Modern CPU with decent specs

- **For embedded tests:**
  - Rust toolchain (1.70+)
  - Cargo

- **For standalone tests:**
  - Go 1.22+
  - Rust toolchain (to build the server)

- **For distributed tests:**
  - Go 1.22+
  - Running VectorXLite cluster (3 nodes)

### Running All Tests

```bash
# Run embedded tests
cd tests/embedded
cargo test

# Run standalone tests
cd tests/standalone
./run_tests.sh

# Run distributed tests (requires cluster to be running)
cd tests/distributed
./run_tests.sh
```

## Test Structure

Each test suite follows best practices:

1. **Isolation:** Tests use unique collection names to avoid conflicts
2. **Cleanup:** Resources are cleaned up after tests
3. **Timeouts:** All tests have appropriate timeouts
4. **Parallel Execution:** Tests can run in parallel where safe
5. **Clear Assertions:** Test failures include descriptive error messages

## CI/CD Integration

All test suites are designed for CI/CD integration:

### Example GitHub Actions Workflow

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  embedded-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run embedded tests
        run: |
          cd tests/embedded
          cargo test --release

  standalone-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-go@v4
        with:
          go-version: '1.22'
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run standalone tests
        run: |
          cd tests/standalone
          ./run_tests.sh

  distributed-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-go@v4
        with:
          go-version: '1.22'
      - name: Setup cluster
        run: |
          # Start cluster nodes
      - name: Run distributed tests
        run: |
          cd tests/distributed
          ./run_tests.sh
```

## Test Coverage Summary

| Test Suite | Coverage | Duration |
|------------|----------|----------|
| Embedded | Core library functionality | ~30s |
| Standalone | gRPC server operations | ~2min |
| Distributed | Cluster operations & replication | ~3-5min |

## Troubleshooting

### Common Issues

1. **Port conflicts**
   - Standalone: Port 50051 must be free
   - Distributed: Ports 5002, 5012, 5022 must be free

2. **Module issues (Go tests)**
   ```bash
   go mod download
   go mod tidy
   ```

3. **Build issues (Rust)**
   ```bash
   cargo clean
   cargo build --release
   ```

4. **Permission denied on scripts**
   ```bash
   chmod +x tests/*/run_tests.sh
   ```

### Getting Help

- Check individual test suite READMEs for detailed troubleshooting
- Review test logs for specific error messages
- Ensure all prerequisites are installed and up to date

## Contributing

When adding new tests:

1. **Choose the right suite:**
   - Core functionality → embedded tests
   - gRPC API → standalone tests
   - Cluster behavior → distributed tests

2. **Follow conventions:**
   - Use descriptive test names
   - Add appropriate documentation
   - Include error messages that help debugging
   - Test both success and failure cases

3. **Test isolation:**
   - Use unique identifiers for test data
   - Clean up resources
   - Don't depend on test execution order

4. **Performance:**
   - Keep tests fast when possible
   - Use appropriate timeouts
   - Consider parallel execution

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    VectorXLite                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Embedded   │  │  Standalone  │  │ Distributed  │ │
│  │   Library    │  │ gRPC Server  │  │   Cluster    │ │
│  │   (Rust)     │  │   (Rust)     │  │    (Go)      │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                 │                  │          │
│         │                 │                  │          │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐ │
│  │ Rust Tests   │  │  Go Client   │  │  Go Client   │ │
│  │  (cargo)     │  │    Tests     │  │    Tests     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## License

These tests are part of the VectorXLite project and follow the same license.

## Related Documentation

- [Project README](../README.md)
- [Embedded Core Documentation](../embedded/core/README.md)
- [Standalone Server Documentation](../standalone/server/README.md)
- [Distributed Cluster Documentation](../distributed/cluster/README.md)

#!/bin/bash

# Standalone Integration Tests Runner
# This script runs integration tests for the standalone gRPC server

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/standalone/server"

echo "========================================="
echo "Standalone Integration Tests"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if server is already running
if lsof -Pi :50051 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo -e "${YELLOW}Server is already running on port 50051${NC}"
    STOP_SERVER=false
else
    echo "Starting standalone server..."
    STOP_SERVER=true

    # Build the server
    cd "$SERVER_DIR"
    cargo build --release

    # Start the server in background
    RUST_LOG=info cargo run --release > /tmp/vectorxlite_server.log 2>&1 &
    SERVER_PID=$!

    # Wait for server to be ready
    echo "Waiting for server to be ready..."
    for i in {1..30}; do
        if lsof -Pi :50051 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
            echo -e "${GREEN}Server is ready!${NC}"
            break
        fi
        sleep 1
    done

    if ! lsof -Pi :50051 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
        echo -e "${RED}Failed to start server${NC}"
        cat /tmp/vectorxlite_server.log
        exit 1
    fi
fi

# Run the tests
echo ""
echo "Running integration tests..."
cd "$SCRIPT_DIR"

# Download dependencies
go mod download

# Run tests with verbose output
if go test -v -timeout 5m ./...; then
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"
    TEST_EXIT=0
else
    echo ""
    echo -e "${RED}✗ Some tests failed${NC}"
    TEST_EXIT=1
fi

# Stop the server if we started it
if [ "$STOP_SERVER" = true ] && [ -n "$SERVER_PID" ]; then
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    echo -e "${GREEN}Server stopped${NC}"
fi

exit $TEST_EXIT

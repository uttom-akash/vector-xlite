#!/bin/bash

# Distributed Cluster Integration Tests Runner
# This script runs integration tests for the distributed cluster

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "========================================="
echo "Distributed Cluster Integration Tests"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cluster is already running
CLUSTER_RUNNING=true
if ! lsof -Pi :5002 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    CLUSTER_RUNNING=false
fi

if [ "$CLUSTER_RUNNING" = false ]; then
    echo -e "${YELLOW}================================================${NC}"
    echo -e "${YELLOW}WARNING: Cluster does not appear to be running${NC}"
    echo -e "${YELLOW}================================================${NC}"
    echo ""
    echo "Please start the cluster manually before running these tests."
    echo ""
    echo "Expected cluster configuration:"
    echo "  - Node 1: localhost:5002 (cluster port)"
    echo "  - Node 2: localhost:5012 (cluster port)"
    echo "  - Node 3: localhost:5022 (cluster port)"
    echo ""
    echo "To start the cluster, use the cluster startup scripts or:"
    echo "  cd $PROJECT_ROOT/distributed/cluster"
    echo "  # Start node 1"
    echo "  # Start node 2"
    echo "  # Start node 3"
    echo ""
    read -p "Press Enter if cluster is running, or Ctrl+C to exit..."
fi

# Verify cluster nodes are accessible
echo "Checking cluster nodes..."
NODES_OK=0
for PORT in 5002 5012 5022; do
    if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1 ; then
        echo -e "${GREEN}✓ Node on port $PORT is accessible${NC}"
        NODES_OK=$((NODES_OK + 1))
    else
        echo -e "${YELLOW}⚠ Node on port $PORT is not accessible${NC}"
    fi
done

if [ $NODES_OK -eq 0 ]; then
    echo -e "${RED}Error: No cluster nodes are accessible${NC}"
    exit 1
fi

echo ""
echo "Found $NODES_OK accessible node(s)"
echo ""

# Run the tests
echo "Running integration tests..."
cd "$SCRIPT_DIR"

# Download dependencies
go mod download

# Run tests with verbose output
if go test -v -timeout 5m ./...; then
    echo ""
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi

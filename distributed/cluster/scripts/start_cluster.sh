#!/bin/bash

# Script to start a 3-node VectorXLite proxy cluster
# Port convention:
#   Node 1: raft=5001, cluster=5002, vector=5003
#   Node 2: raft=5011, cluster=5012, vector=5013
#   Node 3: raft=5021, cluster=5022, vector=5023

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Starting VectorXLite Proxy Cluster ===${NC}"

# Path to VectorXLite gRPC server
VECTOR_XLITE_DIR="/home/akash/Dev/vector-db-rs/standalone/server"

if [ ! -d "$VECTOR_XLITE_DIR" ]; then
    echo -e "${RED}Error: VectorXLite server directory not found at $VECTOR_XLITE_DIR${NC}"
    exit 1
fi

# Create data directory
mkdir -p data logs

# Start 3 VectorXLite servers (one for each node)
echo -e "${YELLOW}Starting VectorXLite servers...${NC}"

# Start VectorXLite server 1 on port 5003
if ! nc -z localhost 5003 2>/dev/null; then
    echo "Starting VectorXLite server 1 on port 5003..."
    (cd "$VECTOR_XLITE_DIR" && cargo run --release -- --port 5003 > ../../distributed/cluster/logs/vector_xlite_node1.log 2>&1) &
    VECTOR_PID1=$!
    echo "$VECTOR_PID1" > .vector_xlite_pids
    echo "  Node1 VectorXLite started with PID: $VECTOR_PID1"
else
    echo -e "${GREEN}VectorXLite server 1 is already running${NC}"
fi

# Start VectorXLite server 2 on port 5013
if ! nc -z localhost 5013 2>/dev/null; then
    echo "Starting VectorXLite server 2 on port 5013..."
    (cd "$VECTOR_XLITE_DIR" && cargo run --release -- --port 5013 > ../../distributed/cluster/logs/vector_xlite_node2.log 2>&1) &
    VECTOR_PID2=$!
    echo "$VECTOR_PID2" >> .vector_xlite_pids
    echo "  Node2 VectorXLite started with PID: $VECTOR_PID2"
else
    echo -e "${GREEN}VectorXLite server 2 is already running${NC}"
fi

# Start VectorXLite server 3 on port 5023
if ! nc -z localhost 5023 2>/dev/null; then
    echo "Starting VectorXLite server 3 on port 5023..."
    (cd "$VECTOR_XLITE_DIR" && cargo run --release -- --port 5023 > ../../distributed/cluster/logs/vector_xlite_node3.log 2>&1) &
    VECTOR_PID3=$!
    echo "$VECTOR_PID3" >> .vector_xlite_pids
    echo "  Node3 VectorXLite started with PID: $VECTOR_PID3"
else
    echo -e "${GREEN}VectorXLite server 3 is already running${NC}"
fi

echo "Waiting for VectorXLite servers to be ready..."
sleep 5

# Wait for all servers to start (max 30 seconds)
for port in 5003 5013 5023; do
    for i in {1..30}; do
        if nc -z localhost $port 2>/dev/null; then
            echo -e "${GREEN}VectorXLite server on port $port is ready!${NC}"
            break
        fi
        sleep 1
    done

    if ! nc -z localhost $port 2>/dev/null; then
        echo -e "${RED}Error: VectorXLite server on port $port failed to start${NC}"
        echo "Check logs/vector_xlite_node*.log for details"
        exit 1
    fi
done

echo ""

# Build the node binary
echo -e "${YELLOW}Building node binary...${NC}"
go build -o bin/node cmd/node/main.go

# Start node1 (bootstrap node)
echo -e "${YELLOW}Starting node1 (bootstrap)...${NC}"
./bin/node \
    -id node1 \
    -port 500 \
    -vector-addr "0.0.0.0:5003" \
    -data-dir ./data \
    -bootstrap \
    > logs/node1.log 2>&1 &
NODE1_PID=$!
echo "Node1 started with PID: $NODE1_PID"
sleep 3

# Start node2
echo -e "${YELLOW}Starting node2...${NC}"
./bin/node \
    -id node2 \
    -port 501 \
    -vector-addr "0.0.0.0:5013" \
    -data-dir ./data \
    > logs/node2.log 2>&1 &
NODE2_PID=$!
echo "Node2 started with PID: $NODE2_PID"
sleep 2

# Start node3
echo -e "${YELLOW}Starting node3...${NC}"
./bin/node \
    -id node3 \
    -port 502 \
    -vector-addr "0.0.0.0:5023" \
    -data-dir ./data \
    > logs/node3.log 2>&1 &
NODE3_PID=$!
echo "Node3 started with PID: $NODE3_PID"
sleep 2

# Save PIDs to file for later cleanup
echo "$NODE1_PID" > .cluster_pids
echo "$NODE2_PID" >> .cluster_pids
echo "$NODE3_PID" >> .cluster_pids

echo -e "${GREEN}All nodes started successfully!${NC}"
echo ""
echo "Cluster configuration:"
echo "  Node1: raft=127.0.0.1:5001, cluster=:5002, vector=:5003"
echo "  Node2: raft=127.0.0.1:5011, cluster=:5012, vector=:5013"
echo "  Node3: raft=127.0.0.1:5021, cluster=:5022, vector=:5023"
echo ""
echo "Logs are in logs/ directory"
echo "To stop cluster: ./scripts/stop_cluster.sh"
echo ""

# Wait a bit for cluster to stabilize
echo -e "${YELLOW}Waiting for cluster to stabilize (5s)...${NC}"
sleep 5

# Build client binary
echo -e "${YELLOW}Building client binary...${NC}"
go build -o bin/client cmd/client/main.go

# Join node2 and node3 to the cluster
echo -e "${YELLOW}Joining node2 to cluster...${NC}"
./bin/client join -addr :5002 -node-id node2 -node-addr 127.0.0.1:5011 || echo "Node2 join failed (may already be member)"

echo -e "${YELLOW}Joining node3 to cluster...${NC}"
./bin/client join -addr :5002 -node-id node3 -node-addr 127.0.0.1:5021 || echo "Node3 join failed (may already be member)"

sleep 2

# Check cluster info
echo -e "${GREEN}=== Cluster Info ===${NC}"
./bin/client info -addr :5002

echo ""
echo -e "${GREEN}Cluster is ready for operations!${NC}"

#!/bin/bash

# Script to stop the VectorXLite proxy cluster and optionally the VectorXLite server

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

STOP_VECTOR_SERVER=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --with-vector-server)
            STOP_VECTOR_SERVER=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--with-vector-server]"
            exit 1
            ;;
    esac
done

echo -e "${YELLOW}Stopping VectorXLite Proxy Cluster...${NC}"

# Stop cluster nodes
if [ -f .cluster_pids ]; then
    while read pid; do
        if kill -0 $pid 2>/dev/null; then
            echo "Stopping cluster node (PID: $pid)..."
            kill $pid
        fi
    done < .cluster_pids
    rm .cluster_pids
    echo -e "${GREEN}Cluster nodes stopped successfully${NC}"
else
    echo "No cluster PIDs file found"
    echo "Attempting to kill any server processes..."
    pkill -f "cmd/server/main.go" || pkill -f "bin/server" || echo "No server processes found"
fi

# Stop VectorXLite servers if requested
if [ "$STOP_VECTOR_SERVER" = true ]; then
    echo -e "${YELLOW}Stopping VectorXLite servers...${NC}"

    if [ -f .vector_xlite_pids ]; then
        while read pid; do
            if kill -0 $pid 2>/dev/null; then
                echo "Stopping VectorXLite server (PID: $pid)..."
                kill $pid
            fi
        done < .vector_xlite_pids
        rm .vector_xlite_pids
        echo -e "${GREEN}VectorXLite servers stopped${NC}"
    else
        echo "No VectorXLite PIDs file found"
        echo "Attempting to kill VectorXLite processes..."
        pkill -f "vector_xlite_grpc" || echo "No VectorXLite processes found"
    fi
else
    echo -e "${YELLOW}VectorXLite servers left running (use --with-vector-server to stop them)${NC}"
fi

echo -e "${GREEN}Done!${NC}"

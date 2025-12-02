#!/bin/bash
set -e

echo "=== Migrating Standalone Mode ==="

# Move Go client
echo "Moving Go client..."
for item in vector_xlite_go_client/*; do
    if [ "$item" != "vector_xlite_go_client/target" ]; then
        git mv "$item" standalone/clients/go/
    fi
done

# Move Go examples
echo "Moving Go examples..."
for item in console_exmples/go_examples/*; do
    git mv "$item" standalone/examples/go/
done

echo "=== Migrating Distributed Mode ==="

# Create distributed structure
echo "Creating distributed structure..."
mkdir -p distributed/cluster distributed/clients distributed/examples distributed/docs

# Move proxy to cluster
echo "Moving cluster proxy..."
for item in vector_xlite_proxy/*; do
    if [ "$item" != "vector_xlite_proxy/target" ] && [ "$item" != "vector_xlite_proxy/data" ] && [ "$item" != "vector_xlite_proxy/logs" ] && [ "$item" != "vector_xlite_proxy/bin" ]; then
        git mv "$item" distributed/cluster/
    fi
done

# Clean up old directories
echo "Cleaning up..."
rmdir vector_xlite_grpc_server 2>/dev/null || true
rmdir vector_xlite_go_client 2>/dev/null || true
rmdir console_exmples/go_examples 2>/dev/null || true
rmdir console_exmples 2>/dev/null || true
rmdir vector_xlite_proxy/target 2>/dev/null || true
rmdir vector_xlite_proxy/data 2>/dev/null || true
rmdir vector_xlite_proxy/logs 2>/dev/null || true
rmdir vector_xlite_proxy/bin 2>/dev/null || true
rmdir vector_xlite_proxy 2>/dev/null || true
rmdir vector_xlite 2>/dev/null || true

echo "=== Migration Complete ==="

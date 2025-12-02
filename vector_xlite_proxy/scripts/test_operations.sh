#!/bin/bash

# Script to test VectorXLite proxy cluster operations

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

CLIENT="./bin/client"
CLUSTER_ADDR=":5002"

echo -e "${GREEN}=== Testing VectorXLite Proxy Cluster ===${NC}"
echo ""

# Test 1: Get cluster info
echo -e "${YELLOW}Test 1: Getting cluster info${NC}"
$CLIENT info -addr $CLUSTER_ADDR
echo ""

# Test 2: Create collection
echo -e "${YELLOW}Test 2: Creating 'users' collection${NC}"
$CLIENT create-collection \
    -addr $CLUSTER_ADDR \
    -name users \
    -dim 4 \
    -schema "create table users (rowid integer primary key, name text, age integer)"
echo ""

# Test 3: Insert vectors
echo -e "${YELLOW}Test 3: Inserting vectors${NC}"
$CLIENT insert \
    -addr $CLUSTER_ADDR \
    -name users \
    -id 1 \
    -vector "1.0,2.0,3.0,4.0" \
    -query "insert into users(name, age) values ('Alice', 25)"

$CLIENT insert \
    -addr $CLUSTER_ADDR \
    -name users \
    -id 2 \
    -vector "2.0,3.0,4.0,5.0" \
    -query "insert into users(name, age) values ('Bob', 30)"

$CLIENT insert \
    -addr $CLUSTER_ADDR \
    -name users \
    -id 3 \
    -vector "1.5,2.5,3.5,4.5" \
    -query "insert into users(name, age) values ('Charlie', 28)"
echo ""

# Wait for replication
echo -e "${YELLOW}Waiting for replication (3s)...${NC}"
sleep 3

# Test 4: Search vectors
echo -e "${YELLOW}Test 4: Searching for similar vectors${NC}"
$CLIENT search \
    -addr $CLUSTER_ADDR \
    -name users \
    -vector "1.0,2.0,3.0,4.0" \
    -k 3 \
    -query "select rowid, name, age from users"
echo ""

# Test 5: Search on different node
echo -e "${YELLOW}Test 5: Searching on node2 (read from follower)${NC}"
$CLIENT search \
    -addr :5012 \
    -name users \
    -vector "2.0,3.0,4.0,5.0" \
    -k 3 \
    -query "select rowid, name, age from users"
echo ""

# Test 6: Try write on follower (should redirect)
echo -e "${YELLOW}Test 6: Testing write redirect (insert on node2)${NC}"
echo "(This should redirect to leader if node2 is not the leader)"
$CLIENT insert \
    -addr :5012 \
    -name users \
    -id 4 \
    -vector "3.0,4.0,5.0,6.0" \
    -query "insert into users(name, age) values ('Dave', 35)" \
    || echo -e "${YELLOW}Write operation handled (may have been redirected)${NC}"
echo ""

echo -e "${GREEN}=== All Tests Completed ===${NC}"

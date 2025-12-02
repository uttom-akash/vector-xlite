# Seed Nodes Topology - Distributed Raft Cluster

## 1. Initial Bootstrap (3-Node Cluster)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Initial Cluster Formation                     │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐         ┌──────────────┐         ┌──────────────┐
    │    Node 1    │◄───────►│    Node 2    │◄───────►│    Node 3    │
    │  (Bootstrap) │  Raft   │  (LEADER)    │  Raft   │  (Follower)  │
    └──────┬───────┘         └──────┬───────┘         └──────┬───────┘
           │                        │                        │
    Raft:  │12001            Raft:  │12002            Raft:  │12003
    HTTP:  │8001             HTTP:  │8002             HTTP:  │8003
           │                        │                        │
           └────────────────────────┴────────────────────────┘
                        Raft Consensus Network
                        (Leader Election, Log Replication)

Config:
  node1.yaml:
    bootstrap: true
    seed_nodes: []

  node2.yaml & node3.yaml:
    bootstrap: false
    seed_nodes: ["127.0.0.1:12001"]
```

---

## 2. New Node Joining via Seed Discovery

```
┌─────────────────────────────────────────────────────────────────┐
│              Node 4 Joins Using Seed Nodes                       │
└─────────────────────────────────────────────────────────────────┘

                           ┌──────────────┐
                           │    Node 4    │
                           │   (NEW)      │
                           └──────┬───────┘
                                  │
                         Raft: 12004
                         HTTP: 8004
                                  │
                      ┌───────────┴───────────┐
                      │   Reads config:       │
                      │   seed_nodes:         │
                      │   - 127.0.0.1:12001   │
                      │   - 127.0.0.1:12002   │
                      └───────────────────────┘

Step 1: Contact Seeds
══════════════════════

    Node 4                    Node 1 (Seed)              Node 2 (Seed)
      │                            │                           │
      │  GET /api/leader          │                           │
      ├───────────────────────────►                           │
      │                            │                           │
      │  {"leader": "127.0.0.1:12002"}                        │
      ◄────────────────────────────┤                           │
      │                            │                           │
      │                                                        │
      │  (Now knows leader is Node 2)                         │
      │                                                        │


Step 2: Request to Join
════════════════════════

    Node 4                                    Node 2 (Leader)
      │                                             │
      │  POST /api/join                            │
      │  {                                          │
      │    "id": "node4",                          │
      │    "addr": "127.0.0.1:12004"              │
      │  }                                          │
      ├─────────────────────────────────────────────►
      │                                             │
      │                                    ┌────────┴────────┐
      │                                    │ raft.AddVoter() │
      │                                    │ ("node4", ...)  │
      │                                    └────────┬────────┘
      │                                             │
      │  {"status": "joined"}                      │
      ◄─────────────────────────────────────────────┤
      │                                             │


Step 3: Config Replication
═══════════════════════════

                    ┌──────────────┐
                    │    Node 2    │
                    │  (LEADER)    │
                    └──────┬───────┘
                           │
          ┌────────────────┼────────────────┐
          │                │                │
          ▼                ▼                ▼
    ┌──────────┐     ┌──────────┐    ┌──────────┐
    │  Node 1  │     │  Node 3  │    │  Node 4  │
    │          │     │          │    │  (NEW)   │
    └──────────┘     └──────────┘    └──────────┘

    All nodes now have updated Raft configuration:
    Servers: [node1, node2, node3, node4]
```

---

## 3. Complete 4-Node Cluster Topology

```
┌─────────────────────────────────────────────────────────────────┐
│                    Final Cluster State                           │
└─────────────────────────────────────────────────────────────────┘

                    ┌──────────────────┐
                    │     Node 2       │
                    │    (LEADER)      │
                    │                  │
                    │ Raft: 12002      │
                    │ HTTP: 8002       │
                    └────────┬─────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
    ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
    │   Node 1    │   │   Node 3    │   │   Node 4    │
    │ (Follower)  │   │ (Follower)  │   │ (Follower)  │
    │             │   │             │   │             │
    │ Raft: 12001 │   │ Raft: 12003 │   │ Raft: 12004 │
    │ HTTP: 8001  │   │ HTTP: 8003  │   │ HTTP: 8004  │
    └──────┬──────┘   └──────┬──────┘   └──────┬──────┘
           │                 │                 │
           └─────────────────┴─────────────────┘
                  Raft Heartbeats & Logs

    ┌────────────────────────────────────────────────┐
    │  VectorXLite FSM Replication                   │
    │  - Collection: "person"                        │
    │  - Vectors: [1.0, 2.0, 3.0, 4.0]              │
    │  - All nodes have identical state              │
    └────────────────────────────────────────────────┘
```

---

## 4. Seed Node Communication Patterns

```
┌─────────────────────────────────────────────────────────────────┐
│                  Two-Layer Communication                         │
└─────────────────────────────────────────────────────────────────┘

Layer 1: HTTP API (Discovery & Join)
═════════════════════════════════════

    New Node                  Seed Nodes
       │                     (Node 1, 2, 3)
       │                           │
       │  GET /api/leader         │   ← Discovery
       ├──────────────────────────►│
       │                           │
       │  POST /api/join          │   ← Join Request
       ├──────────────────────────►│
       │                           │


Layer 2: Raft Protocol (Consensus)
═══════════════════════════════════

    All Nodes (1, 2, 3, 4)
       │
       │  AppendEntries RPC       ← Log Replication
       │  RequestVote RPC         ← Leader Election
       │  InstallSnapshot RPC     ← State Transfer
       │
       └─► TCP Transport (ports 12001-12004)
```

---

## 5. Failure Scenarios

### Scenario A: One Seed Node Down

```
    Node 5 (New)
       │
       │  Try Seed 1 (node1) ──► ✗ Connection Failed
       │
       │  Try Seed 2 (node2) ──► ✓ Success
       │                            │
       │                            └─► Returns leader info
       │
       └─► Joins successfully

    Result: ✓ Cluster accessible (redundant seeds)
```

### Scenario B: All Seeds Down (But Cluster Alive)

```
    Node 5 (New)
       │
       │  Try Seed 1 (node1) ──► ✗ Down
       │  Try Seed 2 (node2) ──► ✗ Down
       │
       └─► ✗ Cannot join (no discovery)

    Cluster: [node3 (LEADER), node4 (Follower)]

    Problem: New nodes can't discover cluster
    Solution: Add more seeds or use dynamic discovery
```

### Scenario C: Leader Failure During Join

```
    Node 5                Node 1 (Seed)         Node 2 (Leader)
       │                        │                      │
       │  GET /api/leader      │                      │
       ├───────────────────────►                      │
       │                        │                      │
       │  "leader: node2"      │                      │
       ◄────────────────────────┤                      │
       │                        │                      │
       │  POST /api/join       │                      ✗ CRASH
       ├──────────────────────────────────────────────►
       │                        │
       │  ✗ Connection timeout │
       │                        │
       │  GET /api/leader ──────►  (New election happened)
       │                        │
       │  "leader: node3"      │
       ◄────────────────────────┤
       │                        │
       │  POST /api/join ───────────────────────────► Node 3 (New Leader)
       │                                                     │
       │  ✓ Joined successfully                             │
       ◄──────────────────────────────────────────────────────┤

    Result: ✓ Retry with new leader
```

---

## 6. Multi-Datacenter Topology (Advanced)

```
┌─────────────────────────────────────────────────────────────────┐
│              Cross-Region Deployment                             │
└─────────────────────────────────────────────────────────────────┘

    Datacenter 1 (US-East)              Datacenter 2 (US-West)
    ════════════════════════            ════════════════════════

    ┌──────────────┐                    ┌──────────────┐
    │    Node 1    │                    │    Node 4    │
    │   (LEADER)   │◄───────────────────►  (Follower)  │
    └──────────────┘   High Latency     └──────────────┘
           │           WAN Link                 │
           │                                     │
    ┌──────────────┐                    ┌──────────────┐
    │    Node 2    │                    │    Node 5    │
    │  (Follower)  │                    │  (Follower)  │
    └──────────────┘                    └──────────────┘
           │                                     │
    ┌──────────────┐                            │
    │    Node 3    │                            │
    │  (Follower)  │                            │
    └──────────────┘                            │

    Seed Config for new nodes:
    seed_nodes:
      - "us-east-1.example.com:12001"   # Node 1
      - "us-east-1.example.com:12002"   # Node 2
      - "us-west-1.example.com:12004"   # Node 4

    Note: DNS names instead of IPs for flexibility
```

---

## 7. Port Mapping Convention

```
┌─────────────────────────────────────────────────────────────────┐
│                     Port Allocation                              │
└─────────────────────────────────────────────────────────────────┘

Node ID    Raft TCP Port    HTTP API Port    VectorXLite gRPC
────────   ──────────────   ─────────────    ─────────────────
node1      12001            8001             50051 (shared)
node2      12002            8002             50051 (shared)
node3      12003            8003             50051 (shared)
node4      12004            8004             50051 (shared)
node5      12005            8005             50051 (shared)

Formula:
  HTTP Port = Raft Port - 4000

Alternative: Use consecutive ports
  Base Port = 10000 + (nodeNum * 10)
  node1: Raft=10010, HTTP=10011, gRPC=10012
  node2: Raft=10020, HTTP=10021, gRPC=10022
```

---

## 8. Configuration Flow

```
┌─────────────────────────────────────────────────────────────────┐
│            How Configuration Flows in System                     │
└─────────────────────────────────────────────────────────────────┘

Static Config (YAML)                 Dynamic Config (Raft)
════════════════════                 ═══════════════════════

┌─────────────────┐                 ┌──────────────────────┐
│  node4.yaml     │                 │  Raft Configuration  │
│  ─────────────  │                 │  ─────────────────   │
│  node_id: node4 │                 │  Servers:            │
│  bind: 12004    │    Join         │    - node1 (Voter)   │
│  seeds:         │─────Request─────►    - node2 (Voter)   │
│    - node1      │                 │    - node3 (Voter)   │
│    - node2      │                 │    - node4 (Voter)   │◄┐
└─────────────────┘                 └──────────────────────┘ │
                                             │                │
        Only used at startup                 │                │
        (to find cluster)                    └────────────────┘
                                          Persisted in BoltDB
                                          (raft-stable.db)
                                          Survives restarts
```

---

## 9. State Transitions

```
┌─────────────────────────────────────────────────────────────────┐
│                 Node Lifecycle States                            │
└─────────────────────────────────────────────────────────────────┘

    ┌─────────┐
    │  START  │
    └────┬────┘
         │
         ├─── Has existing data? ───Yes──► ┌──────────┐
         │                                  │ RESTART  │
         │                                  └────┬─────┘
         │                                       │
         No                                      │
         │                                       │
         ▼                                       │
    Bootstrap?                                   │
         │                                       │
    ├────┴────┐                                 │
    │         │                                  │
   Yes       No                                  │
    │         │                                  │
    ▼         ▼                                  │
┌─────────┐ ┌──────────────┐                   │
│BOOTSTRAP│ │SEED DISCOVERY│                   │
└────┬────┘ └──────┬───────┘                   │
     │             │                             │
     │             ▼                             │
     │      ┌─────────────┐                     │
     │      │ FIND LEADER │                     │
     │      └──────┬──────┘                     │
     │             │                             │
     │             ▼                             │
     │      ┌─────────────┐                     │
     │      │ JOIN REQUEST│                     │
     │      └──────┬──────┘                     │
     │             │                             │
     └─────────────┴─────────────────────────────┘
                   │
                   ▼
          ┌────────────────┐
          │ RAFT FOLLOWER  │◄───────┐
          └────────┬───────┘        │
                   │                 │
                   │ Leader Election │
                   ▼                 │
          ┌────────────────┐        │
          │  RAFT LEADER   │────────┘
          └────────────────┘
```

---

## 10. Real-World Example Commands

```bash
# Terminal 1: Start first node (bootstrap)
$ ./vector_xlite_proxy \
    --node-id=node1 \
    --raft-addr=127.0.0.1:12001 \
    --http-port=8001 \
    --bootstrap

# Terminal 2: Start second node (join via seed)
$ ./vector_xlite_proxy \
    --node-id=node2 \
    --raft-addr=127.0.0.1:12002 \
    --http-port=8002 \
    --seeds=127.0.0.1:12001

# Terminal 3: Start third node
$ ./vector_xlite_proxy \
    --node-id=node3 \
    --raft-addr=127.0.0.1:12003 \
    --http-port=8003 \
    --seeds=127.0.0.1:12001,127.0.0.1:12002

# Later: Start fourth node (uses multiple seeds for redundancy)
$ ./vector_xlite_proxy \
    --node-id=node4 \
    --raft-addr=127.0.0.1:12004 \
    --http-port=8004 \
    --seeds=127.0.0.1:12001,127.0.0.1:12002,127.0.0.1:12003
```

---

## Key Takeaways

1. **Seed nodes** = Known entry points for cluster discovery
2. **HTTP layer** = For discovery and join operations
3. **Raft layer** = For consensus and replication
4. **Two configs** = Static (YAML) for bootstrap, Dynamic (Raft) for runtime
5. **Redundancy** = Multiple seeds prevent single point of failure

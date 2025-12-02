package consensus

import (
	"context"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"time"

	"github.com/hashicorp/raft"
	raftboltdb "github.com/hashicorp/raft-boltdb/v2"
	client "github.com/uttom-akash/vector-xlite/go_grpc_client/client"
)

type VxRaftNode struct {
	id        string
	bindAddr  string
	raft      *raft.Raft
	transport *raft.NetworkTransport
	Fsm       *VxFSM // Exported for access to VectorClient
	dataDir   string
}

// NewRaftNode creates and configures a Raft node (but does not bootstrap).
func NewRaftNode(id, bindAddr, vectorAddr, dataDir string, isInitial bool) (*VxRaftNode, error) {
	if err := os.MkdirAll(dataDir, 0o755); err != nil {
		return nil, fmt.Errorf("failed to create data directory: %w", err)
	}

	config := raft.DefaultConfig()
	config.LocalID = raft.ServerID(id)

	ctx := context.Background()
	vectorClient, err := client.NewClient(ctx, vectorAddr, 5*time.Second)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to VectorXLite at %s: %w", vectorAddr, err)
	}

	fsm := &VxFSM{VectorClient: vectorClient}

	// Set up BoltDB-backed stores
	logStorePath := filepath.Join(dataDir, "raft-log.db")
	stableStorePath := filepath.Join(dataDir, "raft-stable.db")

	logStore, err := raftboltdb.NewBoltStore(logStorePath)
	if err != nil {
		return nil, fmt.Errorf("NewBoltStore log: %w", err)
	}
	stableStore, err := raftboltdb.NewBoltStore(stableStorePath)
	if err != nil {
		return nil, fmt.Errorf("NewBoltStore stable: %w", err)
	}

	snapshotDir := filepath.Join(dataDir, "snapshots")
	snapshotStore, err := raft.NewFileSnapshotStore(snapshotDir, 1, os.Stdout)
	if err != nil {
		return nil, fmt.Errorf("NewFileSnapshotStore: %w", err)
	}

	// Network transport
	addr, err := net.ResolveTCPAddr("tcp", bindAddr)
	if err != nil {
		return nil, fmt.Errorf("ResolveTCPAddr: %w", err)
	}

	transport, err := raft.NewTCPTransport(bindAddr, addr, 3, 10*time.Second, os.Stdout)
	if err != nil {
		return nil, fmt.Errorf("NewTCPTransport: %w", err)
	}

	r, err := raft.NewRaft(config, fsm, logStore, stableStore, snapshotStore, transport)
	if err != nil {
		return nil, fmt.Errorf("NewRaft: %w", err)
	}

	ok := raft.Server{
		ID:      raft.ServerID(id),
		Address: raft.ServerAddress(bindAddr),
	}
	configuration := raft.Configuration{
		Servers: []raft.Server{ok},
	}

	n := &VxRaftNode{
		id:        id,
		bindAddr:  bindAddr,
		raft:      r,
		transport: transport,
		Fsm:       fsm,
		dataDir:   dataDir,
	}

	if isInitial {
		n.raft.BootstrapCluster(configuration)
	}
	return n, nil
}

// BootstrapCluster bootstraps a cluster configuration if the node has no existing state.
func (n *VxRaftNode) BootstrapCluster(peers []NodeInfo) error {
	configuration := raft.Configuration{}
	for _, p := range peers {
		configuration.Servers = append(configuration.Servers, raft.Server{
			ID:      raft.ServerID(p.ID),
			Address: raft.ServerAddress(p.Addr),
		})
	}

	// Only attempt bootstrap if there is no existing state
	hasState := false
	if f := n.raft.Leader(); f != "" {
		hasState = true
	}
	// A safer check is to check log index, but for simplicity we try an AddVoter if not leader.

	if !hasState {
		if err := n.raft.BootstrapCluster(configuration).Error(); err != nil {
			// BootstrapCluster returns an error if cluster is already bootstrapped — ignore that.
			log.Printf("BootstrapCluster returned error (may be benign): %v", err)
		}
	}
	return nil
}

// Apply submits a command to the Raft cluster (must be called on leader for predictable results).
func (n *VxRaftNode) Apply(cmd []byte, timeout time.Duration) raft.ApplyFuture {
	f := n.raft.Apply(cmd, timeout)
	return f
}

func (n *VxRaftNode) Shutdown() raft.Future {
	future := n.raft.Shutdown()
	if n.transport != nil {
		n.transport.Close()
	}
	return future
}

func (n *VxRaftNode) AddVoter(id raft.ServerID, addr raft.ServerAddress, prevIndex uint64, timeout time.Duration) raft.IndexFuture {
	log.Printf("[%s] adding voter: id=%s, addr=%s", n.id, id, addr)

	f := n.raft.AddVoter(raft.ServerID(id), raft.ServerAddress(addr), 0, timeout)

	return f
}

// RemoveServer removes a node from the cluster (must be called on leader).
func (n *VxRaftNode) RemoveServer(id raft.ServerID, prevIndex uint64, timeout time.Duration) raft.IndexFuture {
	log.Printf("[%s] Removing server: id=%s", n.id, id)
	f := n.raft.RemoveServer(raft.ServerID(id), 0, timeout)
	return f
}

// GetConfiguration returns the current cluster configuration.
func (n *VxRaftNode) GetConfiguration() raft.ConfigurationFuture {
	f := n.raft.GetConfiguration()
	return f
}

// PrintConfiguration prints the current cluster configuration.
func (n *VxRaftNode) PrintConfiguration() {
	f := n.GetConfiguration()
	if err := f.Error(); err != nil {
		log.Printf("[%s] Error getting configuration: %v", n.id, err)
		return
	}
	config := f.Configuration()
	log.Printf("[%s] Cluster configuration:", n.id)
	for _, server := range config.Servers {
		log.Printf("  - ID: %s, Address: %s, Suffrage: %v", server.ID, server.Address, server.Suffrage)
	}
}

func (n *VxRaftNode) State() raft.RaftState {
	return n.raft.State()
}
func (n *VxRaftNode) Leader() raft.ServerAddress {
	return n.raft.Leader()
}

package server

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/hashicorp/raft"
	pb "github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/pb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// ClusterNode interface to abstract Raft operations
type ClusterNode interface {
	// Raft operations
	State() raft.RaftState
	Leader() raft.ServerAddress
	AddVoter(id raft.ServerID, address raft.ServerAddress, prevIndex uint64, timeout time.Duration) raft.IndexFuture
	RemoveServer(id raft.ServerID, prevIndex uint64, timeout time.Duration) raft.IndexFuture
	Apply(cmd []byte, timeout time.Duration) raft.ApplyFuture
	GetConfiguration() raft.ConfigurationFuture
}

// ClusterServer implements the ClusterService gRPC service
type ClusterServer struct {
	pb.UnimplementedClusterServiceServer

	raftNode ClusterNode
	nodeID   string
	nodeAddr string

	// Callbacks for write operations
	onCreateCollection func(ctx context.Context, req *pb.CreateCollectionRequest) error
	onInsert           func(ctx context.Context, req *pb.InsertRequest) error
	onDelete           func(ctx context.Context, req *pb.DeleteRequest) error
	onSearch           func(ctx context.Context, req *pb.SearchRequest) (*pb.SearchResponse, error)
}

// ClusterServerConfig holds configuration for the ClusterServer
type ClusterServerConfig struct {
	RaftNode ClusterNode
	// NodeID             string
	// NodeAddr           string
	OnCreateCollection func(ctx context.Context, req *pb.CreateCollectionRequest) error
	OnInsert           func(ctx context.Context, req *pb.InsertRequest) error
	OnDelete           func(ctx context.Context, req *pb.DeleteRequest) error
	OnSearch           func(ctx context.Context, req *pb.SearchRequest) (*pb.SearchResponse, error)
}

// NewClusterServer creates a new ClusterServer instance
func NewClusterServer(cfg ClusterServerConfig) *ClusterServer {
	return &ClusterServer{
		raftNode: cfg.RaftNode,
		// nodeID:             cfg.NodeID,
		// nodeAddr:           cfg.NodeAddr,
		onCreateCollection: cfg.OnCreateCollection,
		onInsert:           cfg.OnInsert,
		onDelete:           cfg.OnDelete,
		onSearch:           cfg.OnSearch,
	}
}

// isLeader checks if this node is the leader
func (s *ClusterServer) isLeader() bool {
	return s.raftNode.State() == raft.Leader
}

// getLeaderAddr returns the current leader address
func (s *ClusterServer) getLeaderAddr() string {
	return string(s.raftNode.Leader())
}

// CreateCollection handles collection creation (write operation)
// Note: Leadership check is handled by LeaderRedirectInterceptor
func (s *ClusterServer) CreateCollection(ctx context.Context, req *pb.CreateCollectionRequest) (*pb.CreateCollectionResponse, error) {
	// Call the registered handler
	if s.onCreateCollection != nil {
		if err := s.onCreateCollection(ctx, req); err != nil {
			return &pb.CreateCollectionResponse{
				Success: false,
				Message: err.Error(),
			}, err
		}
	}

	return &pb.CreateCollectionResponse{
		Success: true,
		Message: "collection created successfully",
	}, nil
}

// Insert handles vector insertion (write operation)
// Note: Leadership check is handled by LeaderRedirectInterceptor
func (s *ClusterServer) Insert(ctx context.Context, req *pb.InsertRequest) (*pb.InsertResponse, error) {
	if s.onInsert != nil {
		if err := s.onInsert(ctx, req); err != nil {
			return &pb.InsertResponse{
				Success: false,
				Message: err.Error(),
			}, err
		}
	}

	return &pb.InsertResponse{
		Success: true,
		Message: "inserted successfully",
	}, nil
}

// Delete handles vector deletion (write operation)
// Note: Leadership check is handled by LeaderRedirectInterceptor
func (s *ClusterServer) Delete(ctx context.Context, req *pb.DeleteRequest) (*pb.DeleteResponse, error) {
	if s.onDelete != nil {
		if err := s.onDelete(ctx, req); err != nil {
			return &pb.DeleteResponse{
				Success: false,
				Message: err.Error(),
			}, err
		}
	}

	return &pb.DeleteResponse{
		Success: true,
		Message: "deleted successfully",
	}, nil
}

// Search handles vector search (read operation - can be handled by any node)
func (s *ClusterServer) Search(ctx context.Context, req *pb.SearchRequest) (*pb.SearchResponse, error) {
	if s.onSearch != nil {
		return s.onSearch(ctx, req)
	}

	return &pb.SearchResponse{Results: []*pb.SearchResultItem{}}, nil
}

// GetClusterInfo returns information about the cluster
func (s *ClusterServer) GetClusterInfo(ctx context.Context, req *pb.GetClusterInfoRequest) (*pb.ClusterInfoResponse, error) {
	// Get Raft configuration
	configFuture := s.raftNode.GetConfiguration()
	if err := configFuture.Error(); err != nil {
		return nil, status.Errorf(codes.Internal, "failed to get configuration: %v", err)
	}

	config := configFuture.Configuration()
	leaderAddr := s.getLeaderAddr()

	// Build node list
	nodes := make([]*pb.NodeInfo, 0, len(config.Servers))
	for _, server := range config.Servers {
		nodeState := "follower"
		if string(server.Address) == leaderAddr {
			nodeState = "leader"
		}

		nodes = append(nodes, &pb.NodeInfo{
			NodeId:  string(server.ID),
			Addr:    string(server.Address),
			State:   nodeState,
			IsVoter: server.Suffrage == raft.Voter,
		})
	}

	return &pb.ClusterInfoResponse{
		LeaderId:   s.findLeaderID(config.Servers, leaderAddr),
		LeaderAddr: leaderAddr,
		Nodes:      nodes,
		State:      s.raftNode.State().String(),
	}, nil
}

// findLeaderID finds the leader ID from server list
func (s *ClusterServer) findLeaderID(servers []raft.Server, leaderAddr string) string {
	for _, server := range servers {
		if string(server.Address) == leaderAddr {
			return string(server.ID)
		}
	}
	return ""
}

// JoinCluster handles node join requests
// Note: Leadership check is handled by LeaderRedirectInterceptor
func (s *ClusterServer) JoinCluster(ctx context.Context, req *pb.JoinClusterRequest) (*pb.JoinClusterResponse, error) {
	log.Printf("Adding node %s at %s to cluster", req.NodeId, req.NodeAddr)

	// Add voter to Raft cluster
	future := s.raftNode.AddVoter(
		raft.ServerID(req.NodeId),
		raft.ServerAddress(req.NodeAddr),
		0,
		10*time.Second,
	)

	if err := future.Error(); err != nil {
		log.Printf("Failed to add node: %v", err)
		return &pb.JoinClusterResponse{
			Success: false,
			Message: fmt.Sprintf("failed to add node: %v", err),
		}, status.Errorf(codes.Internal, "failed to add node: %v", err)
	}

	log.Printf("Successfully added node %s", req.NodeId)

	return &pb.JoinClusterResponse{
		Success:  true,
		Message:  "joined cluster successfully",
		LeaderId: s.nodeID,
	}, nil
}

// LeaveCluster handles node removal requests
// Note: Leadership check is handled by LeaderRedirectInterceptor
func (s *ClusterServer) LeaveCluster(ctx context.Context, req *pb.LeaveClusterRequest) (*pb.LeaveClusterResponse, error) {
	log.Printf("Removing node %s from cluster", req.NodeId)

	// Remove server from Raft cluster
	future := s.raftNode.RemoveServer(
		raft.ServerID(req.NodeId),
		0,
		10*time.Second,
	)

	if err := future.Error(); err != nil {
		log.Printf("Failed to remove node: %v", err)
		return &pb.LeaveClusterResponse{
			Success: false,
			Message: fmt.Sprintf("failed to remove node: %v", err),
		}, status.Errorf(codes.Internal, "failed to remove node: %v", err)
	}

	log.Printf("Successfully removed node %s", req.NodeId)

	return &pb.LeaveClusterResponse{
		Success: true,
		Message: "left cluster successfully",
	}, nil
}

// RegisterWithGRPC registers the ClusterServer with a gRPC server
func (s *ClusterServer) RegisterWithGRPC(grpcServer *grpc.Server) {
	pb.RegisterClusterServiceServer(grpcServer, s)
}

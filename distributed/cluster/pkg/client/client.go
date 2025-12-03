package client

import (
	"context"
	"fmt"
	"time"

	pb "github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/pb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// ClusterClient wraps the gRPC client with convenience methods
type ClusterClient struct {
	conn                *grpc.ClientConn
	client              pb.ClusterServiceClient
	addr                string
	redirectInterceptor *RedirectInterceptor
}

// ClientConfig holds configuration for creating a ClusterClient
type ClientConfig struct {
	// Address of any node in the cluster (seed node)
	Addr string

	// Maximum number of redirects to follow (default: 3)
	MaxRedirects int

	// Enable verbose logging
	VerboseLogging bool

	// Connection timeout (default: 5s)
	ConnectTimeout time.Duration
}

// NewClusterClient creates a new cluster client with automatic leader redirection
func NewClusterClient(cfg ClientConfig) (*ClusterClient, error) {
	if cfg.ConnectTimeout == 0 {
		cfg.ConnectTimeout = 5 * time.Second
	}

	ctx, cancel := context.WithTimeout(context.Background(), cfg.ConnectTimeout)
	defer cancel()

	// Create interceptors
	redirectInterceptor := NewRedirectInterceptor(cfg.MaxRedirects)
	loggingInterceptor := NewLoggingInterceptor(cfg.VerboseLogging)

	// Dial with interceptors
	conn, err := grpc.DialContext(
		ctx,
		cfg.Addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
		grpc.WithChainUnaryInterceptor(
			redirectInterceptor.Unary(),
			loggingInterceptor.Unary(),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to %s: %w", cfg.Addr, err)
	}

	return &ClusterClient{
		conn:                conn,
		client:              pb.NewClusterServiceClient(conn),
		addr:                cfg.Addr,
		redirectInterceptor: redirectInterceptor,
	}, nil
}

// NewClusterClientSimple creates a client with default settings
func NewClusterClientSimple(addr string) (*ClusterClient, error) {
	return NewClusterClient(ClientConfig{
		Addr:           addr,
		MaxRedirects:   3,
		VerboseLogging: false,
		ConnectTimeout: 5 * time.Second,
	})
}

// Close closes the gRPC connection and cleanup resources
func (c *ClusterClient) Close() error {
	if c.redirectInterceptor != nil {
		c.redirectInterceptor.Close()
	}
	return c.conn.Close()
}

// GetAddr returns the initial address this client connected to
func (c *ClusterClient) GetAddr() string {
	return c.addr
}

// ============================================================================
// Write Operations (automatically redirected to leader by interceptor)
// ============================================================================

// CreateCollection creates a new vector collection
func (c *ClusterClient) CreateCollection(ctx context.Context, req *pb.CreateCollectionRequest) (*pb.CreateCollectionResponse, error) {
	return c.client.CreateCollection(ctx, req)
}

// Insert inserts a vector into a collection
func (c *ClusterClient) Insert(ctx context.Context, req *pb.InsertRequest) (*pb.InsertResponse, error) {
	return c.client.Insert(ctx, req)
}

// Delete deletes a vector from a collection
func (c *ClusterClient) Delete(ctx context.Context, req *pb.DeleteRequest) (*pb.DeleteResponse, error) {
	return c.client.Delete(ctx, req)
}

// ============================================================================
// Read Operations (can be served by any node)
// ============================================================================

// Search performs vector similarity search
func (c *ClusterClient) Search(ctx context.Context, req *pb.SearchRequest) (*pb.SearchResponse, error) {
	return c.client.Search(ctx, req)
}

// CollectionExists checks if a collection exists
func (c *ClusterClient) CollectionExists(ctx context.Context, req *pb.CollectionExistsRequest) (*pb.CollectionExistsResponse, error) {
	return c.client.CollectionExists(ctx, req)
}

// ============================================================================
// Cluster Management Operations
// ============================================================================

// GetClusterInfo retrieves cluster information
func (c *ClusterClient) GetClusterInfo(ctx context.Context) (*pb.ClusterInfoResponse, error) {
	return c.client.GetClusterInfo(ctx, &pb.GetClusterInfoRequest{})
}

// JoinCluster requests to join the cluster
func (c *ClusterClient) JoinCluster(ctx context.Context, nodeID, nodeAddr string) (*pb.JoinClusterResponse, error) {
	return c.client.JoinCluster(ctx, &pb.JoinClusterRequest{
		NodeId:   nodeID,
		NodeAddr: nodeAddr,
	})
}

// LeaveCluster requests to leave the cluster
func (c *ClusterClient) LeaveCluster(ctx context.Context, nodeID string) (*pb.LeaveClusterResponse, error) {
	return c.client.LeaveCluster(ctx, &pb.LeaveClusterRequest{
		NodeId: nodeID,
	})
}

// ============================================================================
// Helper Methods
// ============================================================================

// FindLeader returns the leader's address by querying the connected node
func (c *ClusterClient) FindLeader(ctx context.Context) (string, error) {
	info, err := c.GetClusterInfo(ctx)
	if err != nil {
		return "", err
	}
	return info.LeaderAddr, nil
}

// IsLeader checks if the connected node is the leader
func (c *ClusterClient) IsLeader(ctx context.Context) (bool, error) {
	info, err := c.GetClusterInfo(ctx)
	if err != nil {
		return false, err
	}
	return info.State == "Leader", nil
}

// GetNodes returns all nodes in the cluster
func (c *ClusterClient) GetNodes(ctx context.Context) ([]*pb.NodeInfo, error) {
	info, err := c.GetClusterInfo(ctx)
	if err != nil {
		return nil, err
	}
	return info.Nodes, nil
}

package server

import (
	"context"
	"fmt"
	"strings"

	"github.com/hashicorp/raft"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// LeaderRedirectInterceptor handles automatic redirection to leader for write operations
type LeaderRedirectInterceptor struct {
	raftNode ClusterNode
}

// NewLeaderRedirectInterceptor creates a new leader redirect interceptor
func NewLeaderRedirectInterceptor(raftNode ClusterNode) *LeaderRedirectInterceptor {
	return &LeaderRedirectInterceptor{
		raftNode: raftNode,
	}
}

// Unary returns the unary server interceptor
func (i *LeaderRedirectInterceptor) Unary() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		// Check if this is a write operation that requires leadership
		if isWriteOperation(info.FullMethod) {
			// Check if this node is the leader
			if i.raftNode.State() != raft.Leader {
				leaderRaftAddr := string(i.raftNode.Leader())

				// Handle case where no leader is elected yet
				if leaderRaftAddr == "" {
					return nil, status.Errorf(
						codes.Unavailable,
						"no leader available, please retry",
					)
				}

				// Convert raft address (xxx1) to cluster address (xxx2)
				// Example: "127.0.0.1:5001" -> "127.0.0.1:5002"
				leaderClusterAddr, err := convertRaftToClusterAddr(leaderRaftAddr)
				if err != nil {
					return nil, status.Errorf(
						codes.Internal,
						"failed to convert leader address: %v",
						err,
					)
				}

				// Set leader address in response metadata
				md := metadata.Pairs(
					"x-leader-addr", leaderClusterAddr,
					"x-redirect", "true",
				)
				if err := grpc.SetHeader(ctx, md); err != nil {
					return nil, status.Errorf(codes.Internal, "failed to set header: %v", err)
				}

				return nil, status.Errorf(
					codes.FailedPrecondition,
					"not leader, redirect to: %s",
					leaderClusterAddr,
				)
			}
		}

		// This node is leader or it's a read operation - proceed with handler
		return handler(ctx, req)
	}
}

// convertRaftToClusterAddr converts raft address (xxx1) to cluster address (xxx2)
// Example: "127.0.0.1:5001" -> "127.0.0.1:5002"
func convertRaftToClusterAddr(raftAddr string) (string, error) {
	// Split address into host and port
	lastColon := strings.LastIndex(raftAddr, ":")
	if lastColon == -1 {
		return "", fmt.Errorf("invalid address format: %s", raftAddr)
	}

	host := raftAddr[:lastColon]
	port := raftAddr[lastColon+1:]

	// Port should end with '1' for raft addresses
	if len(port) == 0 || port[len(port)-1] != '1' {
		return "", fmt.Errorf("raft address must end with '1': %s", raftAddr)
	}

	// Replace last digit '1' with '2' for cluster port
	clusterPort := port[:len(port)-1] + "2"
	clusterAddr := host + ":" + clusterPort

	return clusterAddr, nil
}

// isWriteOperation checks if the given gRPC method requires leader
func isWriteOperation(method string) bool {
	writeOperations := map[string]bool{
		"/vectorxlite.cluster.ClusterService/CreateCollection": true,
		"/vectorxlite.cluster.ClusterService/Insert":           true,
		"/vectorxlite.cluster.ClusterService/Delete":           true,
		"/vectorxlite.cluster.ClusterService/JoinCluster":      true,
		"/vectorxlite.cluster.ClusterService/LeaveCluster":     true,
	}

	return writeOperations[method]
}

// LoggingInterceptor logs all incoming requests
type LoggingInterceptor struct{}

// NewLoggingInterceptor creates a new logging interceptor
func NewLoggingInterceptor() *LoggingInterceptor {
	return &LoggingInterceptor{}
}

// Unary returns the unary server interceptor for logging
func (i *LoggingInterceptor) Unary() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		fmt.Printf("[gRPC] Method: %s\n", info.FullMethod)

		// Call the handler
		resp, err := handler(ctx, req)

		if err != nil {
			fmt.Printf("[gRPC] Method: %s, Error: %v\n", info.FullMethod, err)
		} else {
			fmt.Printf("[gRPC] Method: %s, Success\n", info.FullMethod)
		}

		return resp, err
	}
}

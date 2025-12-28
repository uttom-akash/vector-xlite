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

type LeaderRedirectInterceptor struct {
	raftNode ClusterNode
}

func NewLeaderRedirectInterceptor(raftNode ClusterNode) *LeaderRedirectInterceptor {
	return &LeaderRedirectInterceptor{
		raftNode: raftNode,
	}
}

func (i *LeaderRedirectInterceptor) Unary() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		if isWriteOperation(info.FullMethod) {
			if i.raftNode.State() != raft.Leader {
				leaderRaftAddr := string(i.raftNode.Leader())
				if leaderRaftAddr == "" {
					return nil, status.Errorf(
						codes.Unavailable,
						"no leader available, please retry",
					)
				}

				leaderClusterAddr, err := convertRaftToClusterAddr(leaderRaftAddr)
				if err != nil {
					return nil, status.Errorf(
						codes.Internal,
						"failed to convert leader address: %v",
						err,
					)
				}

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

		return handler(ctx, req)
	}
}

// convertRaftToClusterAddr: "127.0.0.1:5001" -> "127.0.0.1:5002"
func convertRaftToClusterAddr(raftAddr string) (string, error) {
	lastColon := strings.LastIndex(raftAddr, ":")
	if lastColon == -1 {
		return "", fmt.Errorf("invalid address format: %s", raftAddr)
	}

	host := raftAddr[:lastColon]
	port := raftAddr[lastColon+1:]
	if len(port) == 0 || port[len(port)-1] != '1' {
		return "", fmt.Errorf("raft address must end with '1': %s", raftAddr)
	}

	clusterPort := port[:len(port)-1] + "2"
	clusterAddr := host + ":" + clusterPort

	return clusterAddr, nil
}

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

type LoggingInterceptor struct{}

func NewLoggingInterceptor() *LoggingInterceptor {
	return &LoggingInterceptor{}
}

func (i *LoggingInterceptor) Unary() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		fmt.Printf("[gRPC] Method: %s\n", info.FullMethod)
		resp, err := handler(ctx, req)

		if err != nil {
			fmt.Printf("[gRPC] Method: %s, Error: %v\n", info.FullMethod, err)
		} else {
			fmt.Printf("[gRPC] Method: %s, Success\n", info.FullMethod)
		}

		return resp, err
	}
}

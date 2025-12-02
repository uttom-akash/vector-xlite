package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	types "github.com/uttom-akash/vector-xlite/go_grpc_client/types"
	"github.com/uttom-akash/vector-xlite/vector_xlite_proxy/pkg/consensus"
	"github.com/uttom-akash/vector-xlite/vector_xlite_proxy/pkg/pb"
	"github.com/uttom-akash/vector-xlite/vector_xlite_proxy/pkg/server"
	"google.golang.org/grpc"
)

func main() {
	// CLI flags
	nodeID := flag.String("id", "", "Node ID (e.g., node1)")
	basePort := flag.String("port", "5001", "Base port (xxx1 for raft, xxx2 for cluster, xxx3 for vector_xlite)")
	vectorAddr := flag.String("vector-addr", "0.0.0.0:50051", "VectorXLite gRPC server address")
	dataDir := flag.String("data-dir", "./data", "Data directory for raft logs")
	bootstrap := flag.Bool("bootstrap", false, "Bootstrap as first node in cluster")

	flag.Parse()

	if *nodeID == "" {
		log.Fatal("Node ID is required (use -id flag)")
	}

	// Port convention: xxx1 (raft), xxx2 (cluster grpc), xxx3 (vector xlite if running locally)
	// Base port should be like "500" which becomes "5001" for raft, "5002" for cluster
	raftPort := *basePort + "1"
	clusterPort := *basePort + "2"

	raftAddr := fmt.Sprintf("127.0.0.1:%s", raftPort)
	clusterAddr := fmt.Sprintf(":%s", clusterPort)

	log.Printf("Starting node %s", *nodeID)
	log.Printf("  Raft address: %s", raftAddr)
	log.Printf("  Cluster gRPC address: %s", clusterAddr)
	log.Printf("  VectorXLite address: %s", *vectorAddr)
	log.Printf("  Data directory: %s/%s", *dataDir, *nodeID)
	log.Printf("  Bootstrap: %v", *bootstrap)

	// Create data directory
	nodeDataDir := filepath.Join(*dataDir, *nodeID)
	if err := os.MkdirAll(nodeDataDir, 0755); err != nil {
		log.Fatalf("Failed to create data directory: %v", err)
	}

	// Create raft node
	vxRaftNode, err := consensus.NewRaftNode(*nodeID, raftAddr, *vectorAddr, nodeDataDir, *bootstrap)
	if err != nil {
		log.Fatalf("Failed to create raft node: %v", err)
	}

	// Configure cluster server
	serverCfg := server.ClusterServerConfig{
		RaftNode: vxRaftNode,

		OnCreateCollection: func(ctx context.Context, req *pb.CreateCollectionRequest) error {
			log.Printf("[%s] Creating collection: %s", *nodeID, req.CollectionName)

			collectionConfig, err := types.NewCollectionConfigBuilder().
				CollectionName(req.CollectionName).
				Distance(types.DistanceCosine).
				VectorDimension(req.VectorDimension).
				PayloadTableSchema(req.PayloadTableSchema).
				Build()
			if err != nil {
				return fmt.Errorf("failed to build collection config: %w", err)
			}

			payload, err := json.Marshal(collectionConfig)
			if err != nil {
				return fmt.Errorf("failed to marshal collection config: %w", err)
			}

			comm, err := json.Marshal(consensus.Command{
				Type:    consensus.CmdCreateCollection,
				Payload: payload,
			})
			if err != nil {
				return fmt.Errorf("failed to marshal command: %w", err)
			}

			future := vxRaftNode.Apply(comm, 5*time.Second)
			if err := future.Error(); err != nil {
				log.Printf("[%s] ERROR: Raft Apply failed for CreateCollection: %v", *nodeID, err)
				return fmt.Errorf("raft apply failed: %w", err)
			}

			log.Printf("[%s] Successfully created collection: %s", *nodeID, req.CollectionName)
			return nil
		},

		OnInsert: func(ctx context.Context, req *pb.InsertRequest) error {
			log.Printf("[%s] Inserting into collection: %s", *nodeID, req.CollectionName)

			insertReq, err := types.NewInsertPointBuilder().
				CollectionName(req.CollectionName).
				Id(req.Id).
				Vector(req.Vector).
				PayloadInsertQuery(req.PayloadInsertQuery).
				Build()
			if err != nil {
				return fmt.Errorf("failed to build insert request: %w", err)
			}

			payload, err := json.Marshal(insertReq)
			if err != nil {
				return fmt.Errorf("failed to marshal insert request: %w", err)
			}

			comm, err := json.Marshal(consensus.Command{
				Type:    consensus.CmdInsert,
				Payload: payload,
			})
			if err != nil {
				return fmt.Errorf("failed to marshal command: %w", err)
			}

			future := vxRaftNode.Apply(comm, 5*time.Second)
			if err := future.Error(); err != nil {
				log.Printf("[%s] ERROR: Raft Apply failed for Insert: %v", *nodeID, err)
				return fmt.Errorf("raft apply failed: %w", err)
			}

			log.Printf("[%s] Successfully inserted into collection: %s", *nodeID, req.CollectionName)
			return nil
		},

		OnDelete: func(ctx context.Context, req *pb.DeleteRequest) error {
			log.Printf("[%s] Deleting from collection: %s", *nodeID, req.CollectionName)
			return nil
		},

		OnSearch: func(ctx context.Context, req *pb.SearchRequest) (*pb.SearchResponse, error) {
			log.Printf("[%s] Searching in collection: %s", *nodeID, req.CollectionName)

			builder := types.NewSearchPointBuilder().
				CollectionName(req.CollectionName).
				Vector(req.Vector).
				TopK(req.TopK)

			if req.PayloadSearchQuery != "" {
				builder = builder.PayloadSearchQuery(req.PayloadSearchQuery)
			}

			searchReq, err := builder.Build()
			if err != nil {
				log.Printf("[%s] Failed to build search request: %v", *nodeID, err)
				return &pb.SearchResponse{Results: []*pb.SearchResultItem{}}, fmt.Errorf("failed to build search request: %w", err)
			}

			client := vxRaftNode.Fsm.VectorClient
			resp, err := client.Search(ctx, searchReq)
			if err != nil {
				log.Printf("[%s] Search error: %v", *nodeID, err)
				return &pb.SearchResponse{Results: []*pb.SearchResultItem{}}, fmt.Errorf("search error: %w", err)
			}

			results := make([]*pb.SearchResultItem, 0, len(resp.Results))
			for _, item := range resp.Results {
				payload := make([]*pb.KeyValue, 0, len(item.Payload))
				for _, kv := range item.Payload {
					payload = append(payload, &pb.KeyValue{
						Key:   kv.Key,
						Value: kv.Value,
					})
				}

				resultItem := &pb.SearchResultItem{
					Rowid:    item.Rowid,
					Distance: item.Distance,
					Payload:  payload,
				}
				results = append(results, resultItem)
			}

			return &pb.SearchResponse{Results: results}, nil
		},
	}

	// Create cluster server
	clusterServer := server.NewClusterServer(serverCfg)

	// Create interceptors
	leaderInterceptor := server.NewLeaderRedirectInterceptor(vxRaftNode)
	loggingInterceptor := server.NewLoggingInterceptor()

	// Create gRPC server with interceptors
	grpcServer := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			leaderInterceptor.Unary(),
			loggingInterceptor.Unary(),
		),
	)

	// Register cluster service
	clusterServer.RegisterWithGRPC(grpcServer)

	// Start listening
	lis, err := net.Listen("tcp", clusterAddr)
	if err != nil {
		log.Fatalf("Failed to listen on %s: %v", clusterAddr, err)
	}

	// Start server in goroutine
	go func() {
		log.Printf("[%s] Cluster gRPC server listening on %s", *nodeID, clusterAddr)
		if err := grpcServer.Serve(lis); err != nil {
			log.Fatalf("[%s] Failed to serve: %v", *nodeID, err)
		}
	}()

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	log.Printf("[%s] Shutting down...", *nodeID)
	grpcServer.GracefulStop()
	vxRaftNode.Shutdown()
	log.Printf("[%s] Shutdown complete", *nodeID)
}

package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/client"
	"github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/pb"
)

func main() {
	// Subcommands
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	subcommand := os.Args[1]

	switch subcommand {
	case "create-collection":
		createCollectionCmd()
	case "insert":
		insertCmd()
	case "search":
		searchCmd()
	case "delete":
		deleteCmd()
	case "delete-collection":
		deleteCollectionCmd()
	case "join":
		joinCmd()
	case "info":
		infoCmd()
	default:
		fmt.Printf("Unknown command: %s\n", subcommand)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("VectorXLite Proxy Client CLI")
	fmt.Println()
	fmt.Println("Usage:")
	fmt.Println("  client <command> [flags]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  create-collection  Create a new vector collection")
	fmt.Println("  insert             Insert a vector into a collection")
	fmt.Println("  search             Search for similar vectors")
	fmt.Println("  delete             Delete a vector from a collection")
	fmt.Println("  delete-collection  Delete a collection")
	fmt.Println("  join               Join a node to the cluster")
	fmt.Println("  info               Get cluster information")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  client create-collection -addr :5002 -name users -dim 128 -schema \"create table users(rowid integer primary key, name text)\"")
	fmt.Println("  client insert -addr :5002 -name users -id 1 -vector \"1.0,2.0,3.0\" -query \"insert into users(name) values ('Alice')\"")
	fmt.Println("  client search -addr :5002 -name users -vector \"1.0,2.0,3.0\" -k 5 -query \"select rowid, name from users\"")
	fmt.Println("  client delete -addr :5002 -name users -id 1")
	fmt.Println("  client delete-collection -addr :5002 -name users")
	fmt.Println("  client join -addr :5002 -node-id node2 -node-addr 127.0.0.1:5021")
	fmt.Println("  client info -addr :5002")
}

func createCollectionCmd() {
	fs := flag.NewFlagSet("create-collection", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address (port convention: xxx2)")
	name := fs.String("name", "", "Collection name")
	dim := fs.Int("dim", 128, "Vector dimension")
	schema := fs.String("schema", "", "Payload table schema (SQL CREATE TABLE)")

	fs.Parse(os.Args[2:])

	if *name == "" || *schema == "" {
		log.Fatal("Collection name and schema are required")
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.CreateCollection(ctx, &pb.CreateCollectionRequest{
		CollectionName:     *name,
		Distance:           "Cosine",
		VectorDimension:    uint32(*dim),
		PayloadTableSchema: *schema,
	})
	if err != nil {
		log.Fatalf("Failed to create collection: %v", err)
	}

	fmt.Printf("Success: %v\nMessage: %s\n", resp.Success, resp.Message)
}

func insertCmd() {
	fs := flag.NewFlagSet("insert", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")
	name := fs.String("name", "", "Collection name")
	id := fs.Int64("id", 0, "Vector ID")
	vectorStr := fs.String("vector", "", "Vector values (comma-separated floats)")
	query := fs.String("query", "", "Payload insert SQL query")

	fs.Parse(os.Args[2:])

	if *name == "" || *vectorStr == "" {
		log.Fatal("Collection name and vector are required")
	}

	// Parse vector
	vectorParts := strings.Split(*vectorStr, ",")
	vector := make([]float32, len(vectorParts))
	for i, v := range vectorParts {
		val, err := strconv.ParseFloat(strings.TrimSpace(v), 32)
		if err != nil {
			log.Fatalf("Invalid vector value '%s': %v", v, err)
		}
		vector[i] = float32(val)
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.Insert(ctx, &pb.InsertRequest{
		CollectionName:     *name,
		Id:                 *id,
		Vector:             vector,
		PayloadInsertQuery: *query,
	})
	if err != nil {
		log.Fatalf("Failed to insert: %v", err)
	}

	fmt.Printf("Success: %v\nMessage: %s\n", resp.Success, resp.Message)
}

func searchCmd() {
	fs := flag.NewFlagSet("search", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")
	name := fs.String("name", "", "Collection name")
	vectorStr := fs.String("vector", "", "Query vector (comma-separated floats)")
	k := fs.Int("k", 5, "Number of results (top-K)")
	query := fs.String("query", "", "Payload search SQL query")

	fs.Parse(os.Args[2:])

	if *name == "" || *vectorStr == "" {
		log.Fatal("Collection name and vector are required")
	}

	// Parse vector
	vectorParts := strings.Split(*vectorStr, ",")
	vector := make([]float32, len(vectorParts))
	for i, v := range vectorParts {
		val, err := strconv.ParseFloat(strings.TrimSpace(v), 32)
		if err != nil {
			log.Fatalf("Invalid vector value '%s': %v", v, err)
		}
		vector[i] = float32(val)
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.Search(ctx, &pb.SearchRequest{
		CollectionName:     *name,
		Vector:             vector,
		TopK:               uint32(*k),
		PayloadSearchQuery: *query,
	})
	if err != nil {
		log.Fatalf("Failed to search: %v", err)
	}

	fmt.Printf("Found %d results:\n", len(resp.Results))
	for i, result := range resp.Results {
		fmt.Printf("\n[%d] Rowid: %d, Distance: %.4f\n", i+1, result.Rowid, result.Distance)
		if len(result.Payload) > 0 {
			fmt.Println("  Payload:")
			for _, kv := range result.Payload {
				fmt.Printf("    %s: %s\n", kv.Key, kv.Value)
			}
		}
	}
}

func deleteCmd() {
	fs := flag.NewFlagSet("delete", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")
	name := fs.String("name", "", "Collection name")
	id := fs.Int64("id", 0, "Vector ID to delete")

	fs.Parse(os.Args[2:])

	if *name == "" || *id == 0 {
		log.Fatal("Collection name and id are required")
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.Delete(ctx, &pb.DeleteRequest{
		CollectionName: *name,
		Id:             *id,
	})
	if err != nil {
		log.Fatalf("Failed to delete: %v", err)
	}

	fmt.Printf("Success: %v\nMessage: %s\n", resp.Success, resp.Message)
}

func deleteCollectionCmd() {
	fs := flag.NewFlagSet("delete-collection", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")
	name := fs.String("name", "", "Collection name to delete")

	fs.Parse(os.Args[2:])

	if *name == "" {
		log.Fatal("Collection name is required")
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.DeleteCollection(ctx, &pb.DeleteCollectionRequest{
		CollectionName: *name,
	})
	if err != nil {
		log.Fatalf("Failed to delete collection: %v", err)
	}

	fmt.Printf("Success: %v\nMessage: %s\n", resp.Success, resp.Message)
}

func joinCmd() {
	fs := flag.NewFlagSet("join", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")
	nodeID := fs.String("node-id", "", "New node ID to join")
	nodeAddr := fs.String("node-addr", "", "New node raft address (e.g., 127.0.0.1:5021)")

	fs.Parse(os.Args[2:])

	if *nodeID == "" || *nodeAddr == "" {
		log.Fatal("Node ID and node address are required")
	}

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := clusterClient.JoinCluster(ctx, *nodeID, *nodeAddr)
	if err != nil {
		log.Fatalf("Failed to join node: %v", err)
	}

	fmt.Printf("Success: %v\nMessage: %s\n", resp.Success, resp.Message)
}

func infoCmd() {
	fs := flag.NewFlagSet("info", flag.ExitOnError)
	addr := fs.String("addr", ":5002", "Cluster server address")

	fs.Parse(os.Args[2:])

	clusterClient, err := client.NewClusterClientSimple(*addr)
	if err != nil {
		log.Fatalf("Failed to connect to cluster at %s: %v", *addr, err)
	}
	defer clusterClient.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	info, err := clusterClient.GetClusterInfo(ctx)
	if err != nil {
		log.Fatalf("Failed to get cluster info: %v", err)
	}

	fmt.Printf("Leader ID: %s\n", info.LeaderId)
	fmt.Printf("Leader Address: %s\n", info.LeaderAddr)
	fmt.Printf("Node State: %s\n", info.State)
	fmt.Printf("\nCluster Nodes (%d):\n", len(info.Nodes))
	for _, node := range info.Nodes {
		fmt.Printf("  - %s (%s): %s, Voter: %v\n", node.NodeId, node.Addr, node.State, node.IsVoter)
	}
}

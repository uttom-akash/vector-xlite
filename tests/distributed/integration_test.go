package distributed_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/client"
	"github.com/uttom-akash/vector-xlite/distributed/cluster/pkg/pb"
)

const (
	// Assuming a 3-node cluster is running on these addresses
	node1ClusterAddr = "localhost:5002"
	node2ClusterAddr = "localhost:5012"
	node3ClusterAddr = "localhost:5022"
	dialTimeout      = 5 * time.Second
	requestTimeout   = 10 * time.Second
)

// TestClusterInfo tests retrieving cluster information from all nodes
func TestClusterInfo(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	nodeAddrs := []string{node1ClusterAddr, node2ClusterAddr, node3ClusterAddr}

	for _, addr := range nodeAddrs {
		t.Run(fmt.Sprintf("Node_%s", addr), func(t *testing.T) {
			c, err := client.NewClusterClientSimple(addr)
			if err != nil {
				t.Fatalf("Failed to connect to %s: %v", addr, err)
			}
			defer c.Close()

			info, err := c.GetClusterInfo(ctx)
			if err != nil {
				t.Fatalf("GetClusterInfo failed: %v", err)
			}

			t.Logf("Connected to node at %s", addr)
			t.Logf("Leader ID: %s", info.LeaderId)
			t.Logf("Leader Address: %s", info.LeaderAddr)
			t.Logf("Node State: %s", info.State)
			t.Logf("Cluster has %d nodes", len(info.Nodes))

			// Verify cluster has at least 1 node
			if len(info.Nodes) == 0 {
				t.Error("Cluster should have at least 1 node")
			}

			// Verify there's a leader
			if info.LeaderAddr == "" {
				t.Error("Cluster should have a leader")
			}
		})
	}
}

// TestLeaderElection tests that exactly one node is the leader
func TestLeaderElection(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	nodeAddrs := []string{node1ClusterAddr, node2ClusterAddr, node3ClusterAddr}
	leaderAddrs := make(map[string]int)

	for _, addr := range nodeAddrs {
		c, err := client.NewClusterClientSimple(addr)
		if err != nil {
			t.Logf("Failed to connect to %s (might be down): %v", addr, err)
			continue
		}
		defer c.Close()

		info, err := c.GetClusterInfo(ctx)
		if err != nil {
			t.Logf("GetClusterInfo failed for %s: %v", addr, err)
			continue
		}

		leaderAddrs[info.LeaderAddr]++
	}

	// All nodes should report the same leader
	if len(leaderAddrs) != 1 {
		t.Errorf("Expected all nodes to report same leader, got %d different leaders: %v", len(leaderAddrs), leaderAddrs)
	}

	// The count should equal the number of responding nodes
	for leader, count := range leaderAddrs {
		t.Logf("Leader %s reported by %d nodes", leader, count)
	}
}

// TestCollectionOperations tests creating collections, inserting, and searching in a cluster
func TestCollectionOperations(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Connect to any node (client will auto-redirect to leader for writes)
	c, err := client.NewClusterClientSimple(node1ClusterAddr)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("cluster_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY, name TEXT)", collectionName)

	// Check if collection exists initially
	existsResp, err := c.CollectionExists(ctx, &pb.CollectionExistsRequest{
		CollectionName: collectionName,
	})
	if err != nil {
		t.Fatalf("CollectionExists failed: %v", err)
	}
	if existsResp.Exists {
		t.Fatalf("Collection %s should not exist initially", collectionName)
	}

	// Create collection (will be redirected to leader)
	createResp, err := c.CreateCollection(ctx, &pb.CreateCollectionRequest{
		CollectionName:     collectionName,
		Distance:           "Cosine",
		VectorDimension:    3,
		PayloadTableSchema: schema,
	})
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}
	if !createResp.Success {
		t.Fatalf("CreateCollection not successful: %s", createResp.Message)
	}

	// Verify collection exists
	existsResp, err = c.CollectionExists(ctx, &pb.CollectionExistsRequest{
		CollectionName: collectionName,
	})
	if err != nil {
		t.Fatalf("CollectionExists failed: %v", err)
	}
	if !existsResp.Exists {
		t.Fatalf("Collection %s should exist after creation", collectionName)
	}

	// Insert vectors (will be redirected to leader)
	insertResp, err := c.Insert(ctx, &pb.InsertRequest{
		CollectionName:     collectionName,
		Id:                 1,
		Vector:             []float32{1.0, 2.0, 3.0},
		PayloadInsertQuery: fmt.Sprintf("INSERT INTO %s (name) VALUES ('Alice')", collectionName),
	})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}
	if !insertResp.Success {
		t.Fatalf("Insert not successful: %s", insertResp.Message)
	}

	// Search (can be served by any node)
	searchResp, err := c.Search(ctx, &pb.SearchRequest{
		CollectionName:     collectionName,
		Vector:             []float32{1.0, 2.0, 3.0},
		TopK:               5,
		PayloadSearchQuery: fmt.Sprintf("SELECT rowid, name FROM %s", collectionName),
	})
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if len(searchResp.Results) != 1 {
		t.Fatalf("Expected 1 result, got %d", len(searchResp.Results))
	}

	if searchResp.Results[0].Rowid != 1 {
		t.Errorf("Expected rowid 1, got %d", searchResp.Results[0].Rowid)
	}

	// Verify payload
	hasName := false
	for _, kv := range searchResp.Results[0].Payload {
		if kv.Key == "name" && kv.Value == "Alice" {
			hasName = true
		}
	}
	if !hasName {
		t.Error("Expected payload to contain name='Alice'")
	}
}

// TestWriteRedirection tests that writes to followers are redirected to leader
func TestWriteRedirection(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Find the leader first
	leaderClient, err := client.NewClusterClientSimple(node1ClusterAddr)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer leaderClient.Close()

	info, err := leaderClient.GetClusterInfo(ctx)
	if err != nil {
		t.Fatalf("GetClusterInfo failed: %v", err)
	}

	leaderAddr := info.LeaderAddr
	t.Logf("Leader is at: %s", leaderAddr)

	// Find a follower node
	var followerAddr string
	nodeAddrs := []string{node1ClusterAddr, node2ClusterAddr, node3ClusterAddr}
	for _, addr := range nodeAddrs {
		if !contains(leaderAddr, addr) {
			followerAddr = addr
			break
		}
	}

	if followerAddr == "" {
		t.Skip("No follower found, single-node cluster")
	}

	t.Logf("Using follower at: %s", followerAddr)

	// Connect to follower
	followerClient, err := client.NewClusterClientSimple(followerAddr)
	if err != nil {
		t.Fatalf("Failed to connect to follower: %v", err)
	}
	defer followerClient.Close()

	collectionName := fmt.Sprintf("redirect_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Try to create collection through follower (should be redirected)
	createResp, err := followerClient.CreateCollection(ctx, &pb.CreateCollectionRequest{
		CollectionName:     collectionName,
		Distance:           "Cosine",
		VectorDimension:    3,
		PayloadTableSchema: schema,
	})
	if err != nil {
		t.Fatalf("CreateCollection through follower failed: %v", err)
	}
	if !createResp.Success {
		t.Fatalf("CreateCollection not successful: %s", createResp.Message)
	}

	// Verify the collection exists by querying the leader
	existsResp, err := leaderClient.CollectionExists(ctx, &pb.CollectionExistsRequest{
		CollectionName: collectionName,
	})
	if err != nil {
		t.Fatalf("CollectionExists failed: %v", err)
	}
	if !existsResp.Exists {
		t.Error("Collection should exist after creation through follower")
	}
}

// TestReadFromFollowers tests that reads can be served by any node
func TestReadFromFollowers(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Connect to first node to create collection and insert data
	c, err := client.NewClusterClientSimple(node1ClusterAddr)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("read_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Create collection
	_, err = c.CreateCollection(ctx, &pb.CreateCollectionRequest{
		CollectionName:     collectionName,
		Distance:           "Cosine",
		VectorDimension:    3,
		PayloadTableSchema: schema,
	})
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Insert data
	_, err = c.Insert(ctx, &pb.InsertRequest{
		CollectionName: collectionName,
		Id:             1,
		Vector:         []float32{1.0, 0.0, 0.0},
	})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}

	// Wait for replication
	time.Sleep(2 * time.Second)

	// Try to search from all nodes
	nodeAddrs := []string{node1ClusterAddr, node2ClusterAddr, node3ClusterAddr}
	successCount := 0

	for _, addr := range nodeAddrs {
		t.Run(fmt.Sprintf("Search_from_%s", addr), func(t *testing.T) {
			nodeClient, err := client.NewClusterClientSimple(addr)
			if err != nil {
				t.Logf("Failed to connect to %s (might be down): %v", addr, err)
				return
			}
			defer nodeClient.Close()

			searchResp, err := nodeClient.Search(ctx, &pb.SearchRequest{
				CollectionName: collectionName,
				Vector:         []float32{1.0, 0.0, 0.0},
				TopK:           5,
			})
			if err != nil {
				t.Errorf("Search from %s failed: %v", addr, err)
				return
			}

			if len(searchResp.Results) != 1 {
				t.Errorf("Expected 1 result from %s, got %d", addr, len(searchResp.Results))
				return
			}

			successCount++
		})
	}

	if successCount == 0 {
		t.Error("No nodes were able to serve the search request")
	}
}

// TestConcurrentWrites tests concurrent write operations to the cluster
func TestConcurrentWrites(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	c, err := client.NewClusterClientSimple(node1ClusterAddr)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("concurrent_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Create collection
	_, err = c.CreateCollection(ctx, &pb.CreateCollectionRequest{
		CollectionName:     collectionName,
		Distance:           "Cosine",
		VectorDimension:    3,
		PayloadTableSchema: schema,
	})
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Concurrent inserts
	const numGoroutines = 10
	const vectorsPerGoroutine = 10
	errChan := make(chan error, numGoroutines)

	for g := 0; g < numGoroutines; g++ {
		go func(goroutineID int) {
			for i := 0; i < vectorsPerGoroutine; i++ {
				id := int64(goroutineID*vectorsPerGoroutine + i + 1)
				_, err := c.Insert(ctx, &pb.InsertRequest{
					CollectionName: collectionName,
					Id:             id,
					Vector:         []float32{float32(id), float32(id + 1), float32(id + 2)},
				})
				if err != nil {
					errChan <- fmt.Errorf("insert id %d: %w", id, err)
					return
				}
			}
			errChan <- nil
		}(g)
	}

	// Wait for all goroutines
	for i := 0; i < numGoroutines; i++ {
		if err := <-errChan; err != nil {
			t.Errorf("Goroutine error: %v", err)
		}
	}

	// Wait for replication
	time.Sleep(2 * time.Second)

	// Verify all vectors were inserted
	searchResp, err := c.Search(ctx, &pb.SearchRequest{
		CollectionName: collectionName,
		Vector:         []float32{1.0, 2.0, 3.0},
		TopK:           200,
	})
	if err != nil {
		t.Fatalf("Final search failed: %v", err)
	}

	expectedCount := numGoroutines * vectorsPerGoroutine
	if len(searchResp.Results) != expectedCount {
		t.Errorf("Expected %d results, got %d", expectedCount, len(searchResp.Results))
	}
}

// TestMultipleCollectionsInCluster tests creating and managing multiple collections
func TestMultipleCollectionsInCluster(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClusterClientSimple(node1ClusterAddr)
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer c.Close()

	baseTime := time.Now().UnixNano()
	collections := []string{
		fmt.Sprintf("collection_1_%d", baseTime),
		fmt.Sprintf("collection_2_%d", baseTime),
		fmt.Sprintf("collection_3_%d", baseTime),
	}

	// Create all collections
	for _, name := range collections {
		schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", name)
		_, err := c.CreateCollection(ctx, &pb.CreateCollectionRequest{
			CollectionName:     name,
			Distance:           "Cosine",
			VectorDimension:    3,
			PayloadTableSchema: schema,
		})
		if err != nil {
			t.Fatalf("Failed to create collection %s: %v", name, err)
		}
	}

	// Verify all exist
	for _, name := range collections {
		existsResp, err := c.CollectionExists(ctx, &pb.CollectionExistsRequest{
			CollectionName: name,
		})
		if err != nil {
			t.Fatalf("CollectionExists failed for %s: %v", name, err)
		}
		if !existsResp.Exists {
			t.Errorf("Collection %s should exist", name)
		}
	}

	// Insert data into each and verify isolation
	for i, name := range collections {
		_, err := c.Insert(ctx, &pb.InsertRequest{
			CollectionName: name,
			Id:             int64(i + 1),
			Vector:         []float32{float32(i), float32(i + 1), float32(i + 2)},
		})
		if err != nil {
			t.Fatalf("Insert into %s failed: %v", name, err)
		}
	}

	// Verify each collection has only its own data
	for i, name := range collections {
		searchResp, err := c.Search(ctx, &pb.SearchRequest{
			CollectionName: name,
			Vector:         []float32{float32(i), float32(i + 1), float32(i + 2)},
			TopK:           10,
		})
		if err != nil {
			t.Fatalf("Search in %s failed: %v", name, err)
		}

		if len(searchResp.Results) != 1 {
			t.Errorf("Collection %s should have 1 result, got %d", name, len(searchResp.Results))
		}
	}
}

// Helper function
func contains(s, substr string) bool {
	return len(s) > 0 && len(substr) > 0 && (s == substr || len(s) >= len(substr) && s[len(s)-len(substr):] == substr)
}

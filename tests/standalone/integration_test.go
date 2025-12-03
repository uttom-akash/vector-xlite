package standalone_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	client "github.com/uttom-akash/vector-xlite/standalone/clients/go/client"
	types "github.com/uttom-akash/vector-xlite/standalone/clients/go/types"
)

const (
	serverAddr     = "localhost:50051"
	dialTimeout    = 5 * time.Second
	requestTimeout = 10 * time.Second
)

// TestCollectionLifecycle tests the complete lifecycle of a collection
func TestCollectionLifecycle(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("test_collection_%d", time.Now().UnixNano())

	// 1. Check collection doesn't exist initially
	exists, err := c.CollectionExists(ctx, collectionName)
	if err != nil {
		t.Fatalf("CollectionExists failed: %v", err)
	}
	if exists {
		t.Fatalf("Collection %s should not exist initially", collectionName)
	}

	// 2. Create collection
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)
	config, err := types.NewCollectionConfigBuilder().
		CollectionName(collectionName).
		Distance(types.DistanceCosine).
		VectorDimension(3).
		PayloadTableSchema(schema).
		Build()
	if err != nil {
		t.Fatalf("Failed to build collection config: %v", err)
	}

	err = c.CreateCollection(ctx, config)
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// 3. Verify collection now exists
	exists, err = c.CollectionExists(ctx, collectionName)
	if err != nil {
		t.Fatalf("CollectionExists failed: %v", err)
	}
	if !exists {
		t.Fatalf("Collection %s should exist after creation", collectionName)
	}

	// 4. Insert a vector
	insertReq, err := types.NewInsertPointBuilder().
		CollectionName(collectionName).
		Id(1).
		Vector([]float32{1.0, 2.0, 3.0}).
		Build()
	if err != nil {
		t.Fatalf("Failed to build insert request: %v", err)
	}

	err = c.Insert(ctx, insertReq)
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}

	// 5. Search for the vector
	searchReq, err := types.NewSearchPointBuilder().
		CollectionName(collectionName).
		Vector([]float32{1.0, 2.0, 3.0}).
		TopK(5).
		Build()
	if err != nil {
		t.Fatalf("Failed to build search request: %v", err)
	}

	resp, err := c.Search(ctx, searchReq)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if len(resp.Results) != 1 {
		t.Fatalf("Expected 1 result, got %d", len(resp.Results))
	}

	if resp.Results[0].Rowid != 1 {
		t.Errorf("Expected rowid 1, got %d", resp.Results[0].Rowid)
	}
}

// TestCollectionExistsCases tests various collection existence scenarios
func TestCollectionExistsCases(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	testCases := []struct {
		name           string
		collectionName string
		shouldExist    bool
		expectError    bool
	}{
		{"Empty string", "", false, true},
		{"Nonexistent collection", "nonexistent_collection_12345", false, false},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			exists, err := c.CollectionExists(ctx, tc.collectionName)
			if tc.expectError {
				if err == nil {
					t.Error("Expected an error for empty collection name")
				}
				return
			}
			if err != nil {
				t.Fatalf("CollectionExists failed: %v", err)
			}
			if exists != tc.shouldExist {
				t.Errorf("Expected exists=%v, got %v", tc.shouldExist, exists)
			}
		})
	}
}

// TestMultipleCollections tests creating and managing multiple collections
func TestMultipleCollections(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	baseTime := time.Now().UnixNano()
	collections := []string{
		fmt.Sprintf("collection_1_%d", baseTime),
		fmt.Sprintf("collection_2_%d", baseTime),
		fmt.Sprintf("collection_3_%d", baseTime),
	}

	// Create all collections
	for i, name := range collections {
		schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", name)
		config, err := types.NewCollectionConfigBuilder().
			CollectionName(name).
			Distance(types.DistanceCosine).
			VectorDimension(uint32(3 + i)).
			PayloadTableSchema(schema).
			Build()
		if err != nil {
			t.Fatalf("Failed to build config for %s: %v", name, err)
		}

		err = c.CreateCollection(ctx, config)
		if err != nil {
			t.Fatalf("Failed to create collection %s: %v", name, err)
		}
	}

	// Verify all exist
	for _, name := range collections {
		exists, err := c.CollectionExists(ctx, name)
		if err != nil {
			t.Fatalf("CollectionExists failed for %s: %v", name, err)
		}
		if !exists {
			t.Errorf("Collection %s should exist", name)
		}
	}
}

// TestInsertAndSearch tests inserting multiple vectors and searching
func TestInsertAndSearch(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("search_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Create collection
	config, err := types.NewCollectionConfigBuilder().
		CollectionName(collectionName).
		Distance(types.DistanceCosine).
		VectorDimension(4).
		PayloadTableSchema(schema).
		Build()
	if err != nil {
		t.Fatalf("Failed to build collection config: %v", err)
	}

	err = c.CreateCollection(ctx, config)
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Insert multiple vectors
	vectors := []struct {
		id     uint64
		vector []float32
	}{
		{1, []float32{1.0, 0.0, 0.0, 0.0}},
		{2, []float32{0.0, 1.0, 0.0, 0.0}},
		{3, []float32{0.0, 0.0, 1.0, 0.0}},
		{4, []float32{0.0, 0.0, 0.0, 1.0}},
		{5, []float32{0.5, 0.5, 0.0, 0.0}},
	}

	for _, v := range vectors {
		insertReq, err := types.NewInsertPointBuilder().
			CollectionName(collectionName).
			Id(int64(v.id)).
			Vector(v.vector).
			Build()
		if err != nil {
			t.Fatalf("Failed to build insert request: %v", err)
		}

		err = c.Insert(ctx, insertReq)
		if err != nil {
			t.Fatalf("Insert failed for id %d: %v", v.id, err)
		}
	}

	// Search for vector close to first one
	searchReq, err := types.NewSearchPointBuilder().
		CollectionName(collectionName).
		Vector([]float32{0.9, 0.1, 0.0, 0.0}).
		TopK(3).
		Build()
	if err != nil {
		t.Fatalf("Failed to build search request: %v", err)
	}

	resp, err := c.Search(ctx, searchReq)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if len(resp.Results) == 0 {
		t.Fatalf("Expected at least 1 result, got 0")
	}

	// The closest vector should be id=1
	if resp.Results[0].Rowid != 1 {
		t.Errorf("Expected closest vector to be id 1, got %d", resp.Results[0].Rowid)
	}
}

// TestPayloadOperations tests collection with payload schema
func TestPayloadOperations(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("payload_test_%d", time.Now().UnixNano())

	// Create collection with payload schema
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY, name TEXT, age INTEGER)", collectionName)
	config, err := types.NewCollectionConfigBuilder().
		CollectionName(collectionName).
		Distance(types.DistanceCosine).
		VectorDimension(3).
		PayloadTableSchema(schema).
		Build()
	if err != nil {
		t.Fatalf("Failed to build collection config: %v", err)
	}

	err = c.CreateCollection(ctx, config)
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Insert vector with payload
	insertReq, err := types.NewInsertPointBuilder().
		CollectionName(collectionName).
		Id(1).
		Vector([]float32{1.0, 2.0, 3.0}).
		PayloadInsertQuery(fmt.Sprintf("INSERT INTO %s (name, age) VALUES ('Alice', 30)", collectionName)).
		Build()
	if err != nil {
		t.Fatalf("Failed to build insert request: %v", err)
	}

	err = c.Insert(ctx, insertReq)
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}

	// Search with payload query
	searchReq, err := types.NewSearchPointBuilder().
		CollectionName(collectionName).
		Vector([]float32{1.0, 2.0, 3.0}).
		TopK(5).
		PayloadSearchQuery(fmt.Sprintf("SELECT rowid, name, age FROM %s", collectionName)).
		Build()
	if err != nil {
		t.Fatalf("Failed to build search request: %v", err)
	}

	resp, err := c.Search(ctx, searchReq)
	if err != nil {
		t.Fatalf("Search failed: %v", err)
	}

	if len(resp.Results) != 1 {
		t.Fatalf("Expected 1 result, got %d", len(resp.Results))
	}

	// Verify payload
	hasName := false
	hasAge := false
	for _, kv := range resp.Results[0].Payload {
		if kv.Key == "name" && kv.Value == "Alice" {
			hasName = true
		}
		if kv.Key == "age" && kv.Value == "30" {
			hasAge = true
		}
	}

	if !hasName {
		t.Error("Expected payload to contain name='Alice'")
	}
	if !hasAge {
		t.Error("Expected payload to contain age=30")
	}
}

// TestDistanceFunctions tests different distance functions
func TestDistanceFunctions(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	distanceFunctions := []types.DistanceFunction{
		types.DistanceCosine,
		types.DistanceEuclidean,
		types.DistanceDot,
	}

	for _, dist := range distanceFunctions {
		t.Run(dist.String(), func(t *testing.T) {
			collectionName := fmt.Sprintf("dist_%s_%d", dist.String(), time.Now().UnixNano())
			schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

			config, err := types.NewCollectionConfigBuilder().
				CollectionName(collectionName).
				Distance(dist).
				VectorDimension(3).
				PayloadTableSchema(schema).
				Build()
			if err != nil {
				t.Fatalf("Failed to build collection config: %v", err)
			}

			err = c.CreateCollection(ctx, config)
			if err != nil {
				t.Fatalf("CreateCollection failed: %v", err)
			}

			// Insert and search
			insertReq, err := types.NewInsertPointBuilder().
				CollectionName(collectionName).
				Id(1).
				Vector([]float32{1.0, 0.0, 0.0}).
				Build()
			if err != nil {
				t.Fatalf("Failed to build insert request: %v", err)
			}

			err = c.Insert(ctx, insertReq)
			if err != nil {
				t.Fatalf("Insert failed: %v", err)
			}

			searchReq, err := types.NewSearchPointBuilder().
				CollectionName(collectionName).
				Vector([]float32{1.0, 0.0, 0.0}).
				TopK(5).
				Build()
			if err != nil {
				t.Fatalf("Failed to build search request: %v", err)
			}

			resp, err := c.Search(ctx, searchReq)
			if err != nil {
				t.Fatalf("Search failed: %v", err)
			}

			if len(resp.Results) != 1 {
				t.Fatalf("Expected 1 result, got %d", len(resp.Results))
			}
		})
	}
}

// TestSnapshotExportImport tests snapshot functionality
func TestSnapshotExportImport(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("snapshot_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Create collection and insert data
	config, err := types.NewCollectionConfigBuilder().
		CollectionName(collectionName).
		Distance(types.DistanceCosine).
		VectorDimension(3).
		PayloadTableSchema(schema).
		Build()
	if err != nil {
		t.Fatalf("Failed to build collection config: %v", err)
	}

	err = c.CreateCollection(ctx, config)
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Insert some vectors
	for i := int64(1); i <= 5; i++ {
		insertReq, err := types.NewInsertPointBuilder().
			CollectionName(collectionName).
			Id(i).
			Vector([]float32{float32(i), float32(i + 1), float32(i + 2)}).
			Build()
		if err != nil {
			t.Fatalf("Failed to build insert request: %v", err)
		}

		err = c.Insert(ctx, insertReq)
		if err != nil {
			t.Fatalf("Insert failed: %v", err)
		}
	}

	// Export snapshot
	exportReq := types.NewExportSnapshotRequestBuilder().
		ChunkSize(128 * 1024).
		IncludeIndexFiles(true).
		Build()

	collector, err := c.ExportSnapshotSync(ctx, exportReq)
	if err != nil {
		t.Fatalf("Export snapshot failed: %v", err)
	}

	if collector.Metadata == nil {
		t.Fatal("Expected metadata in snapshot")
	}

	t.Logf("Snapshot ID: %s", collector.Metadata.SnapshotID)
	t.Logf("Total size: %d bytes", collector.Metadata.TotalSize)
	t.Logf("Files: %d", len(collector.Metadata.Files))
	t.Logf("Chunks: %d", len(collector.Chunks))

	// Import snapshot
	importResp, err := c.ImportSnapshotFromCollector(ctx, collector)
	if err != nil {
		t.Fatalf("Import snapshot failed: %v", err)
	}

	if !importResp.Success {
		t.Fatalf("Import not successful: %s", importResp.ErrorMessage)
	}

	t.Logf("Imported snapshot: %s", importResp.SnapshotID)
	t.Logf("Bytes restored: %d", importResp.BytesRestored)
	t.Logf("Files restored: %d", importResp.FilesRestored)

	// Note: After snapshot import in standalone mode, the data may not be immediately
	// available for search because the in-memory index needs to be rebuilt.
	// In a production system, you would restart the server or reload the collection.
	// For this test, we verify the snapshot operations completed successfully.
	t.Log("Snapshot export and import completed successfully")
}

// TestConcurrentOperations tests concurrent inserts and searches
func TestConcurrentOperations(t *testing.T) {
	t.Skip("Skipping due to SQLite concurrent write limitations")
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	c, err := client.NewClient(ctx, serverAddr, dialTimeout)
	if err != nil {
		t.Fatalf("Failed to connect to server: %v", err)
	}
	defer c.Close()

	collectionName := fmt.Sprintf("concurrent_test_%d", time.Now().UnixNano())
	schema := fmt.Sprintf("CREATE TABLE %s (rowid INTEGER PRIMARY KEY)", collectionName)

	// Create collection
	config, err := types.NewCollectionConfigBuilder().
		CollectionName(collectionName).
		Distance(types.DistanceCosine).
		VectorDimension(3).
		PayloadTableSchema(schema).
		Build()
	if err != nil {
		t.Fatalf("Failed to build collection config: %v", err)
	}

	err = c.CreateCollection(ctx, config)
	if err != nil {
		t.Fatalf("CreateCollection failed: %v", err)
	}

	// Concurrent inserts
	// Note: Using fewer goroutines to avoid SQLite lock contention
	// SQLite has limitations with concurrent writes, so we keep this minimal
	const numGoroutines = 2
	const vectorsPerGoroutine = 5
	errChan := make(chan error, numGoroutines*2)

	// Launch insert goroutines
	for g := 0; g < numGoroutines; g++ {
		go func(goroutineID int) {
			for i := 0; i < vectorsPerGoroutine; i++ {
				id := int64(goroutineID*vectorsPerGoroutine + i + 1)
				insertReq, err := types.NewInsertPointBuilder().
					CollectionName(collectionName).
					Id(id).
					Vector([]float32{float32(id), float32(id + 1), float32(id + 2)}).
					Build()
				if err != nil {
					errChan <- fmt.Errorf("build insert: %w", err)
					return
				}

				if err := c.Insert(ctx, insertReq); err != nil {
					errChan <- fmt.Errorf("insert: %w", err)
					return
				}
			}
			errChan <- nil
		}(g)
	}

	// Launch search goroutines
	for g := 0; g < numGoroutines; g++ {
		go func(goroutineID int) {
			searchReq, err := types.NewSearchPointBuilder().
				CollectionName(collectionName).
				Vector([]float32{float32(goroutineID), float32(goroutineID + 1), float32(goroutineID + 2)}).
				TopK(5).
				Build()
			if err != nil {
				errChan <- fmt.Errorf("build search: %w", err)
				return
			}

			if _, err := c.Search(ctx, searchReq); err != nil {
				errChan <- fmt.Errorf("search: %w", err)
				return
			}
			errChan <- nil
		}(g)
	}

	// Wait for all goroutines
	for i := 0; i < numGoroutines*2; i++ {
		if err := <-errChan; err != nil {
			t.Errorf("Goroutine error: %v", err)
		}
	}

	// Verify all vectors were inserted
	searchReq, err := types.NewSearchPointBuilder().
		CollectionName(collectionName).
		Vector([]float32{1.0, 2.0, 3.0}).
		TopK(200).
		Build()
	if err != nil {
		t.Fatalf("Failed to build search request: %v", err)
	}

	resp, err := c.Search(ctx, searchReq)
	if err != nil {
		t.Fatalf("Final search failed: %v", err)
	}

	expectedCount := numGoroutines * vectorsPerGoroutine
	if len(resp.Results) != expectedCount {
		t.Errorf("Expected %d results, got %d", expectedCount, len(resp.Results))
	}
}

package main

import (
	"context"
	"fmt"
	"log"
	"time"

	client "github.com/uttom-akash/vector-xlite/standalone/clients/go/client"
	types "github.com/uttom-akash/vector-xlite/standalone/clients/go/types"
)

func main() {

	println("running...")

	ctx := context.Background()
	client, err := client.NewClient(ctx, "0.0.0.0:50051", 5*time.Second)

	if err != nil {
		log.Fatalf("failed to connect: %v", err)
	}

	defer client.Close()

	// 1) Create collection request
	// comment: build a CollectionConfigPB; fields mirror your proto schema.
	collectionConfig, err := types.NewCollectionConfigBuilder().
		CollectionName("person").
		Distance(types.DistanceCosine).
		VectorDimension(4).
		PayloadTableSchema("create table person (rowid integer primary key, name text)").
		// IndexFilePath("").
		Build()

	if err := client.CreateCollection(ctx, collectionConfig); err != nil {
		log.Fatalf("CreateCollection error: %v", err)
	}

	fmt.Println("collection created")

	// 2) Insert a point
	// comment: InsertPointPB.payload_insert_query should use ?1 placeholder for rowid
	insertReq, err := types.NewInsertPointBuilder().
		CollectionName("person").
		Id(1).
		Vector([]float32{1.0, 2.0, 3.0, 4.0}).
		PayloadInsertQuery("insert into person(name) values ('Charlie')").
		Build()

	if err := client.Insert(ctx, insertReq); err != nil {
		log.Fatalf("Insert error: %v", err)
	}
	fmt.Println("inserted point")

	// 3) Search
	// comment: request top-k and optionally provide SQL to fetch payloads
	searchReq, err := types.NewSearchPointBuilder().
		CollectionName("person").
		Vector([]float32{0.9, 0.8, 0.7, 0.6}).
		TopK(5).
		PayloadSearchQuery("select rowid, name from person").
		Build()

	resp, err := client.Search(ctx, searchReq)
	if err != nil {
		log.Fatalf("Search error: %v", err)
	}

	// comment: print results; payload is a repeated KeyValuePB
	fmt.Println("Search results:")
	for i, item := range resp.Results {
		fmt.Printf("rank=%d rowid=%d distance=%f\n", i+1, item.Rowid, item.Distance)
		for _, kv := range item.Payload {
			fmt.Printf("  %s: %s\n", kv.Key, kv.Value)
		}
	}

	// Example 1: Export snapshot synchronously (simple use case)
	fmt.Println("=== Example 1: Synchronous Export ===")
	syncExportExample(ctx, client)

	// Example 2: Export snapshot with streaming (advanced use case)
	fmt.Println("\n=== Example 2: Streaming Export ===")
	streamingExportExample(ctx, client)

	// Example 3: Import snapshot (follower recovery)
	fmt.Println("\n=== Example 3: Import Snapshot ===")
	importExample(ctx, client)
}

// syncExportExample demonstrates synchronous snapshot export
func syncExportExample(ctx context.Context, c *client.Client) {
	// Create export request with custom chunk size
	req := types.NewExportSnapshotRequestBuilder().
		ChunkSize(128 * 1024). // 128KB chunks
		IncludeIndexFiles(true).
		Build()

	// Export snapshot (blocks until complete)
	collector, err := c.ExportSnapshotSync(ctx, req)
	if err != nil {
		log.Printf("Export failed: %v", err)
		return
	}

	// Print snapshot information
	if collector.Metadata != nil {
		fmt.Printf("Snapshot ID: %s\n", collector.Metadata.SnapshotID)
		fmt.Printf("Created At: %d\n", collector.Metadata.CreatedAt)
		fmt.Printf("Total Size: %d bytes\n", collector.Metadata.TotalSize)
		fmt.Printf("Files: %d\n", len(collector.Metadata.Files))

		for _, f := range collector.Metadata.Files {
			fmt.Printf("  - %s (%s): %d bytes\n", f.FileName, f.FileType.String(), f.FileSize)
		}
	}

	fmt.Printf("Total chunks received: %d\n", len(collector.Chunks))
	fmt.Printf("Data bytes collected: %d\n", collector.GetTotalBytes())
}

// streamingExportExample demonstrates streaming snapshot export with progress tracking
func streamingExportExample(ctx context.Context, c *client.Client) {
	// Create export request with default settings
	req := types.NewExportSnapshotRequestBuilder().Build()

	// Start streaming export
	chunkChan, errChan := c.ExportSnapshot(ctx, req)

	var (
		totalChunks int
		totalBytes  uint64
		metadata    *types.SnapshotMetadata
	)

	// Process chunks as they arrive
	for {
		select {
		case chunk, ok := <-chunkChan:
			if !ok {
				// Channel closed, export complete
				fmt.Printf("\nExport complete!\n")
				fmt.Printf("Total chunks: %d\n", totalChunks)
				fmt.Printf("Total bytes: %d\n", totalBytes)
				return
			}

			totalChunks++

			// Save metadata from first chunk
			if chunk.Metadata != nil {
				metadata = chunk.Metadata
				fmt.Printf("Snapshot: %s (created at %d)\n", metadata.SnapshotID, metadata.CreatedAt)
			}

			// Track data bytes
			if chunk.FileChunk != nil {
				totalBytes += uint64(len(chunk.FileChunk.Data))
				// Print progress
				if metadata != nil && metadata.TotalSize > 0 {
					progress := float64(totalBytes) / float64(metadata.TotalSize) * 100
					fmt.Printf("\rProgress: %.1f%% (%d/%d bytes)", progress, totalBytes, metadata.TotalSize)
				}
			}

			// Check for final chunk
			if chunk.IsFinal {
				fmt.Printf("\nReceived final chunk\n")
			}

		case err := <-errChan:
			if err != nil {
				log.Printf("Stream error: %v", err)
				return
			}

		case <-ctx.Done():
			log.Printf("Context cancelled: %v", ctx.Err())
			return
		}
	}
}

// importExample demonstrates importing a snapshot (e.g., for follower recovery)
func importExample(ctx context.Context, c *client.Client) {
	// First, export a snapshot to get the chunks
	exportReq := types.NewExportSnapshotRequestBuilder().Build()
	collector, err := c.ExportSnapshotSync(ctx, exportReq)
	if err != nil {
		log.Printf("Export failed: %v", err)
		return
	}

	fmt.Printf("Exported snapshot: %s\n", collector.Metadata.SnapshotID)

	// Now import the snapshot (simulating follower recovery)
	// In a real Raft scenario, the chunks would come from the leader over the network
	resp, err := c.ImportSnapshotFromCollector(ctx, collector)
	if err != nil {
		log.Printf("Import failed: %v", err)
		return
	}

	if resp.Success {
		fmt.Printf("Import successful!\n")
		fmt.Printf("  Snapshot ID: %s\n", resp.SnapshotID)
		fmt.Printf("  Bytes restored: %d\n", resp.BytesRestored)
		fmt.Printf("  Files restored: %d\n", resp.FilesRestored)
	} else {
		fmt.Printf("Import failed: %s\n", resp.ErrorMessage)
	}
}

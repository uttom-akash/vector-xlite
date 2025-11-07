package main

import (
	"context"
	"fmt"
	"log"
	"time"

	client "github.com/uttom-akash/vector-xlite/go_grpc_client/client"
	types "github.com/uttom-akash/vector-xlite/go_grpc_client/types"
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
}

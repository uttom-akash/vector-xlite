package client

import (
	"context"
	"errors"
	"fmt"
	"time"

	pb "github.com/uttom-akash/vector-xlite/standalone/clients/go/pb"
	types "github.com/uttom-akash/vector-xlite/standalone/clients/go/types"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type Client struct {
	conn     *grpc.ClientConn
	pbClient pb.VectorXLitePBClient
}

// NewClient dials the server address and returns a Client.
// Uses insecure transport for local testing; update for production.
func NewClient(ctx context.Context, addr string, dialTimeout time.Duration) (*Client, error) {
	ctxDial, cancel := context.WithTimeout(ctx, dialTimeout)
	defer cancel()
	conn, err := grpc.DialContext(ctxDial, addr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
	if err != nil {
		return nil, fmt.Errorf("dial grpc: %w", err)
	}
	return &Client{
		conn:     conn,
		pbClient: pb.NewVectorXLitePBClient(conn),
	}, nil
}

// Close closes the underlying grpc connection.
func (c *Client) Close() error {
	if c.conn == nil {
		return nil
	}
	return c.conn.Close()
}

// CreateCollection sends a CreateCollection request to the server.
func (c *Client) CreateCollection(ctx context.Context, cfg *types.CollectionConfig) error {
	if cfg == nil {
		return errors.New("nil config")
	}
	pbCfg := &pb.CollectionConfigPB{
		CollectionName:     cfg.CollectionName,
		Distance:           cfg.Distance.String(),
		VectorDimension:    cfg.VectorDimension,
		PayloadTableSchema: cfg.PayloadTableSchema,
		IndexFilePath:      cfg.IndexFilePath,
	}
	_, err := c.pbClient.CreateCollection(ctx, pbCfg)
	return err
}

// Insert sends an InsertPoint request to the server.
func (c *Client) Insert(ctx context.Context, p *types.InsertPoint) error {
	if p == nil {
		return errors.New("nil point")
	}
	pbPt := &pb.InsertPointPB{
		CollectionName:     p.CollectionName,
		Id:                 p.Id,
		Vector:             p.Vector,
		PayloadInsertQuery: p.PayloadInsertQuery,
	}
	_, err := c.pbClient.Insert(ctx, pbPt)
	return err
}

// Delete sends a Delete request to the server.
func (c *Client) Delete(ctx context.Context, collectionName string, id int64) (*pb.DeleteResponsePB, error) {
	if collectionName == "" {
		return nil, errors.New("collection name cannot be empty")
	}
	pbReq := &pb.DeleteRequestPB{
		CollectionName: collectionName,
		Id:             id,
	}
	return c.pbClient.Delete(ctx, pbReq)
}

// DeleteCollection sends a DeleteCollection request to the server.
func (c *Client) DeleteCollection(ctx context.Context, collectionName string) (*pb.DeleteResponsePB, error) {
	if collectionName == "" {
		return nil, errors.New("collection name cannot be empty")
	}
	pbReq := &pb.DeleteCollectionRequestPB{
		CollectionName: collectionName,
	}
	return c.pbClient.DeleteCollection(ctx, pbReq)
}


// Search sends a SearchPoint request and converts the response.
func (c *Client) Search(ctx context.Context, q *types.SearchPoint) (*types.SearchResponse, error) {
	if q == nil {
		return nil, errors.New("nil search point")
	}
	pbReq := &pb.SearchPointPB{
		CollectionName:     q.CollectionName,
		Vector:             q.Vector,
		TopK:               q.TopK,
		PayloadSearchQuery: q.PayloadSearchQuery,
	}
	pbResp, err := c.pbClient.Search(ctx, pbReq)
	if err != nil {
		return nil, err
	}
	resp := &types.SearchResponse{Results: make([]types.SearchResultItem, 0, len(pbResp.Results))}
	for _, r := range pbResp.Results {
		item := types.SearchResultItem{
			Rowid:    r.Rowid,
			Distance: r.Distance,
			Payload:  make([]types.KeyValue, 0, len(r.Payload)),
		}
		for _, kv := range r.Payload {
			item.Payload = append(item.Payload, types.KeyValue{Key: kv.Key, Value: kv.Value})
		}
		resp.Results = append(resp.Results, item)
	}
	return resp, nil
}

// CollectionExists checks if a collection with the given name exists.
func (c *Client) CollectionExists(ctx context.Context, collectionName string) (bool, error) {
	if collectionName == "" {
		return false, errors.New("collection name cannot be empty")
	}
	pbReq := &pb.CollectionExistsRequestPB{
		CollectionName: collectionName,
	}
	pbResp, err := c.pbClient.CollectionExists(ctx, pbReq)
	if err != nil {
		return false, err
	}
	return pbResp.Exists, nil
}

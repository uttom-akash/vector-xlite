package client

import (
	"context"
	"errors"
	"io"

	pb "github.com/uttom-akash/vector-xlite/go_grpc_client/pb"
	types "github.com/uttom-akash/vector-xlite/go_grpc_client/types"
)

// ExportSnapshot exports a consistent snapshot of the database.
// It returns a channel that streams snapshot chunks for Raft FSM integration.
// The caller should range over the channel to receive all chunks.
func (c *Client) ExportSnapshot(ctx context.Context, req *types.ExportSnapshotRequest) (<-chan *types.SnapshotChunk, <-chan error) {
	chunkChan := make(chan *types.SnapshotChunk, 32)
	errChan := make(chan error, 1)

	go func() {
		defer close(chunkChan)
		defer close(errChan)

		// Build protobuf request
		pbReq := &pb.ExportSnapshotRequestPB{}
		if req != nil {
			pbReq.ChunkSize = req.ChunkSize
			pbReq.IncludeIndexFiles = req.IncludeIndexFiles
		}

		// Call the streaming RPC
		stream, err := c.pbClient.ExportSnapshot(ctx, pbReq)
		if err != nil {
			errChan <- err
			return
		}

		// Receive chunks from the stream
		for {
			pbChunk, err := stream.Recv()
			if err == io.EOF {
				return
			}
			if err != nil {
				errChan <- err
				return
			}

			// Convert protobuf chunk to types.SnapshotChunk
			chunk := convertPbChunkToType(pbChunk)
			chunkChan <- chunk

			// Check if this is the final chunk
			if chunk.IsFinal {
				return
			}
		}
	}()

	return chunkChan, errChan
}

// ExportSnapshotSync exports a snapshot and collects all chunks synchronously.
// This is a convenience method for simple use cases.
func (c *Client) ExportSnapshotSync(ctx context.Context, req *types.ExportSnapshotRequest) (*types.SnapshotCollector, error) {
	collector := types.NewSnapshotCollector()

	chunkChan, errChan := c.ExportSnapshot(ctx, req)

	for {
		select {
		case chunk, ok := <-chunkChan:
			if !ok {
				// Channel closed, check for completeness
				if !collector.IsComplete() {
					return nil, errors.New("snapshot incomplete: did not receive final chunk")
				}
				return collector, nil
			}
			collector.AddChunk(chunk)

		case err := <-errChan:
			if err != nil {
				return nil, err
			}

		case <-ctx.Done():
			return nil, ctx.Err()
		}
	}
}

// ImportSnapshot imports a snapshot from a slice of chunks.
// This is used by Raft followers to restore state from a leader's snapshot.
func (c *Client) ImportSnapshot(ctx context.Context, chunks []*types.SnapshotChunk) (*types.ImportSnapshotResponse, error) {
	if len(chunks) == 0 {
		return nil, errors.New("no chunks to import")
	}

	// Start the streaming RPC
	stream, err := c.pbClient.ImportSnapshot(ctx)
	if err != nil {
		return nil, err
	}

	// Send all chunks
	for _, chunk := range chunks {
		pbChunk := convertTypeChunkToPb(chunk)
		if err := stream.Send(pbChunk); err != nil {
			return nil, err
		}
	}

	// Close the stream and receive the response
	pbResp, err := stream.CloseAndRecv()
	if err != nil {
		return nil, err
	}

	// Convert response
	return &types.ImportSnapshotResponse{
		Success:       pbResp.Success,
		ErrorMessage:  pbResp.ErrorMessage,
		SnapshotID:    pbResp.SnapshotId,
		BytesRestored: pbResp.BytesRestored,
		FilesRestored: pbResp.FilesRestored,
	}, nil
}

// ImportSnapshotFromCollector imports a snapshot from a collector.
// This is a convenience method to use with ExportSnapshotSync.
func (c *Client) ImportSnapshotFromCollector(ctx context.Context, collector *types.SnapshotCollector) (*types.ImportSnapshotResponse, error) {
	if collector == nil {
		return nil, errors.New("nil collector")
	}

	// Convert chunks slice
	chunks := make([]*types.SnapshotChunk, len(collector.Chunks))
	for i := range collector.Chunks {
		chunks[i] = &collector.Chunks[i]
	}

	return c.ImportSnapshot(ctx, chunks)
}

// convertPbChunkToType converts a protobuf SnapshotChunkPB to types.SnapshotChunk
func convertPbChunkToType(pbChunk *pb.SnapshotChunkPB) *types.SnapshotChunk {
	chunk := &types.SnapshotChunk{
		Sequence: pbChunk.Sequence,
		IsFinal:  pbChunk.IsFinal,
	}

	// Convert metadata if present
	if pbChunk.Metadata != nil {
		chunk.Metadata = &types.SnapshotMetadata{
			SnapshotID: pbChunk.Metadata.SnapshotId,
			CreatedAt:  pbChunk.Metadata.CreatedAt,
			TotalSize:  pbChunk.Metadata.TotalSize,
			Version:    pbChunk.Metadata.Version,
			Checksum:   pbChunk.Metadata.Checksum,
			Files:      make([]types.SnapshotFileInfo, 0, len(pbChunk.Metadata.Files)),
		}

		for _, f := range pbChunk.Metadata.Files {
			chunk.Metadata.Files = append(chunk.Metadata.Files, types.SnapshotFileInfo{
				FileName: f.FileName,
				FileType: convertPbFileType(pb.SnapshotFileTypePB(f.FileType)),
				FileSize: f.FileSize,
				Checksum: f.Checksum,
			})
		}
	}

	// Convert file chunk if present
	if pbChunk.FileChunk != nil {
		chunk.FileChunk = &types.FileChunk{
			FileName:    pbChunk.FileChunk.FileName,
			Offset:      pbChunk.FileChunk.Offset,
			Data:        pbChunk.FileChunk.Data,
			IsLastChunk: pbChunk.FileChunk.IsLastChunk,
		}
	}

	return chunk
}

// convertTypeChunkToPb converts types.SnapshotChunk to protobuf SnapshotChunkPB
func convertTypeChunkToPb(chunk *types.SnapshotChunk) *pb.SnapshotChunkPB {
	pbChunk := &pb.SnapshotChunkPB{
		Sequence: chunk.Sequence,
		IsFinal:  chunk.IsFinal,
	}

	// Convert metadata if present
	if chunk.Metadata != nil {
		pbChunk.Metadata = &pb.SnapshotMetadataPB{
			SnapshotId: chunk.Metadata.SnapshotID,
			CreatedAt:  chunk.Metadata.CreatedAt,
			TotalSize:  chunk.Metadata.TotalSize,
			Version:    chunk.Metadata.Version,
			Checksum:   chunk.Metadata.Checksum,
			Files:      make([]*pb.SnapshotFileInfoPB, 0, len(chunk.Metadata.Files)),
		}

		for _, f := range chunk.Metadata.Files {
			pbChunk.Metadata.Files = append(pbChunk.Metadata.Files, &pb.SnapshotFileInfoPB{
				FileName: f.FileName,
				FileType: convertTypeFileTypeToPb(f.FileType),
				FileSize: f.FileSize,
				Checksum: f.Checksum,
			})
		}
	}

	// Convert file chunk if present
	if chunk.FileChunk != nil {
		pbChunk.FileChunk = &pb.SnapshotFilePB{
			FileName:    chunk.FileChunk.FileName,
			Offset:      chunk.FileChunk.Offset,
			Data:        chunk.FileChunk.Data,
			IsLastChunk: chunk.FileChunk.IsLastChunk,
		}
	}

	return pbChunk
}

// convertPbFileType converts protobuf file type to types.SnapshotFileType
func convertPbFileType(pbType pb.SnapshotFileTypePB) types.SnapshotFileType {
	switch pbType {
	case pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_SQLITE_DB:
		return types.SnapshotFileTypeSqliteDB
	case pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_HNSW_INDEX:
		return types.SnapshotFileTypeHnswIndex
	case pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_WAL:
		return types.SnapshotFileTypeWal
	default:
		return types.SnapshotFileTypeUnknown
	}
}

// convertTypeFileTypeToPb converts types.SnapshotFileType to protobuf
func convertTypeFileTypeToPb(t types.SnapshotFileType) pb.SnapshotFileTypePB {
	switch t {
	case types.SnapshotFileTypeSqliteDB:
		return pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_SQLITE_DB
	case types.SnapshotFileTypeHnswIndex:
		return pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_HNSW_INDEX
	case types.SnapshotFileTypeWal:
		return pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_WAL
	default:
		return pb.SnapshotFileTypePB_SNAPSHOT_FILE_TYPE_UNKNOWN
	}
}

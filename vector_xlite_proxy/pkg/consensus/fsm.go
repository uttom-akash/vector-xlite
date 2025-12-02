package consensus

import (
	"context"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"log"

	"github.com/hashicorp/raft"
	client "github.com/uttom-akash/vector-xlite/go_grpc_client/client"
	types "github.com/uttom-akash/vector-xlite/go_grpc_client/types"
)

type VxFSM struct {
	VectorClient *client.Client // Exported for external access
}

// Apply applies a Raft log entry to the VectorXLite database
func (f *VxFSM) Apply(rlog *raft.Log) interface{} {
	var cmd Command
	if err := json.Unmarshal(rlog.Data, &cmd); err != nil {
		log.Fatalf("Failed to deserialize raft log in Apply method, err: %v", err)
		return &ApplyResult{Error: err}
	}

	ctx := context.Background()
	switch cmd.Type {
	case CmdCreateCollection:
		return f.applyCreateCollection(ctx, cmd.Payload)
	case CmdInsert:
		return f.applyInsert(ctx, cmd.Payload)
	default:
		log.Default()
		return &ApplyResult{Error: nil}
	}
}

func (f *VxFSM) applyCreateCollection(ctx context.Context, payload json.RawMessage) interface{} {

	var collectionConfig types.CollectionConfig

	if err := json.Unmarshal(payload, &collectionConfig); err != nil {
		return &ApplyResult{Error: err}
	}

	f.VectorClient.CreateCollection(ctx, &collectionConfig)

	return &ApplyResult{}
}

func (f *VxFSM) applyInsert(ctx context.Context, payload json.RawMessage) interface{} {
	var insertPoint types.InsertPoint

	log.Printf("Insert in")
	if err := json.Unmarshal(payload, &insertPoint); err != nil {
		log.Fatalf("failed deserialize the insert point, err: %v", err)
		return &ApplyResult{Error: err}
	}

	log.Printf("insert : %v", insertPoint)

	f.VectorClient.Insert(ctx, &insertPoint)

	return &ApplyResult{}
}

// Snapshot returns an FSMSnapshot for Raft snapshot support
func (f *VxFSM) Snapshot() (raft.FSMSnapshot, error) {
	ctx := context.Background()

	// Export snapshot from VectorXLite
	req := types.NewExportSnapshotRequestBuilder().
		ChunkSize(256 * 1024).
		IncludeIndexFiles(true).
		Build()

	collector, err := f.VectorClient.ExportSnapshotSync(ctx, req)
	if err != nil {
		return nil, err
	}

	return &VectorXLiteSnapshot{collector: collector}, nil
}

// Restore restores the FSM from a snapshot
func (f *VxFSM) Restore(rc io.ReadCloser) error {
	// Read all chunks from the snapshot reader
	chunks, err := readSnapshotChunks(rc)
	if err != nil {
		return err
	}

	ctx := context.Background()

	_, err = f.VectorClient.ImportSnapshot(ctx, chunks)

	return err
}

// VectorXLiteSnapshot implements raft.FSMSnapshot
type VectorXLiteSnapshot struct {
	collector *types.SnapshotCollector
}

func (s *VectorXLiteSnapshot) Persist(sink raft.SnapshotSink) error {
	// Write snapshot data to sink

	if err := writeSnapshotToSink(sink, s.collector); err != nil {
		sink.Cancel()
		return err
	}
	return sink.Close()
}

func (s *VectorXLiteSnapshot) Release() {}

// writeChunk writes a single snapshot chunk with length prefix to a writer.
// Format: [4-byte length (uint32)][JSON-encoded chunk data]
func writeChunk(w io.Writer, chunk *types.SnapshotChunk) error {
	// Marshal chunk to JSON
	data, err := json.Marshal(chunk)
	if err != nil {
		return fmt.Errorf("failed to marshal chunk: %w", err)
	}

	// Write length prefix (4 bytes, big endian)
	length := uint32(len(data))
	if err := binary.Write(w, binary.BigEndian, length); err != nil {
		return fmt.Errorf("failed to write length prefix: %w", err)
	}

	// Write JSON data
	if _, err := w.Write(data); err != nil {
		return fmt.Errorf("failed to write chunk data: %w", err)
	}

	return nil
}

// readChunk reads a single snapshot chunk with length prefix from a reader.
// Returns io.EOF when no more chunks are available.
func readChunk(r io.Reader) (*types.SnapshotChunk, error) {
	// Read length prefix (4 bytes, big endian)
	var length uint32
	if err := binary.Read(r, binary.BigEndian, &length); err != nil {
		return nil, err // io.EOF is expected at end of stream
	}

	// Read chunk data
	data := make([]byte, length)
	if _, err := io.ReadFull(r, data); err != nil {
		return nil, fmt.Errorf("failed to read chunk data: %w", err)
	}

	// Unmarshal JSON to SnapshotChunk
	var chunk types.SnapshotChunk
	if err := json.Unmarshal(data, &chunk); err != nil {
		return nil, fmt.Errorf("failed to unmarshal chunk: %w", err)
	}

	return &chunk, nil
}

// writeSnapshotToSink writes all snapshot chunks from a collector to a Raft snapshot sink.
func writeSnapshotToSink(sink raft.SnapshotSink, collector *types.SnapshotCollector) error {
	for i, chunk := range collector.Chunks {
		if err := writeChunk(sink, &chunk); err != nil {
			return fmt.Errorf("failed to write chunk %d: %w", i, err)
		}
	}
	return nil
}

// readSnapshotChunks reads all snapshot chunks from an io.ReadCloser.
// The reader is closed before returning.
func readSnapshotChunks(rc io.ReadCloser) ([]*types.SnapshotChunk, error) {
	defer rc.Close()

	chunks := make([]*types.SnapshotChunk, 0)

	for {
		chunk, err := readChunk(rc)
		if err == io.EOF {
			// Expected end of stream
			break
		}
		if err != nil {
			return nil, fmt.Errorf("failed to read chunk: %w", err)
		}

		chunks = append(chunks, chunk)

		// Optional: Break if we've read the final chunk
		if chunk.IsFinal {
			break
		}
	}

	return chunks, nil
}

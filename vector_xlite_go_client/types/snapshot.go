package types

import "errors"

// SnapshotFileType represents the type of file in a snapshot
type SnapshotFileType int

const (
	SnapshotFileTypeUnknown SnapshotFileType = iota
	SnapshotFileTypeSqliteDB
	SnapshotFileTypeHnswIndex
	SnapshotFileTypeWal
)

// String returns the string representation of the file type
func (t SnapshotFileType) String() string {
	switch t {
	case SnapshotFileTypeSqliteDB:
		return "SqliteDB"
	case SnapshotFileTypeHnswIndex:
		return "HnswIndex"
	case SnapshotFileTypeWal:
		return "Wal"
	default:
		return "Unknown"
	}
}

// ExportSnapshotRequest represents a request to export a snapshot
type ExportSnapshotRequest struct {
	// ChunkSize specifies the size of each chunk in bytes (default: 64KB)
	ChunkSize uint32
	// IncludeIndexFiles specifies whether to include HNSW index files
	IncludeIndexFiles bool
}

// ExportSnapshotRequestBuilder builds an ExportSnapshotRequest
type ExportSnapshotRequestBuilder struct {
	req ExportSnapshotRequest
}

// NewExportSnapshotRequestBuilder creates a new builder with defaults
func NewExportSnapshotRequestBuilder() *ExportSnapshotRequestBuilder {
	return &ExportSnapshotRequestBuilder{
		req: ExportSnapshotRequest{
			ChunkSize:         64 * 1024, // 64KB default
			IncludeIndexFiles: true,
		},
	}
}

// ChunkSize sets the chunk size in bytes
func (b *ExportSnapshotRequestBuilder) ChunkSize(size uint32) *ExportSnapshotRequestBuilder {
	b.req.ChunkSize = size
	return b
}

// IncludeIndexFiles sets whether to include index files
func (b *ExportSnapshotRequestBuilder) IncludeIndexFiles(include bool) *ExportSnapshotRequestBuilder {
	b.req.IncludeIndexFiles = include
	return b
}

// Build returns the built request
func (b *ExportSnapshotRequestBuilder) Build() *ExportSnapshotRequest {
	return &b.req
}

// SnapshotFileInfo contains information about a file in the snapshot
type SnapshotFileInfo struct {
	FileName string
	FileType SnapshotFileType
	FileSize uint64
	Checksum string
}

// SnapshotMetadata contains metadata about the entire snapshot
type SnapshotMetadata struct {
	SnapshotID string
	CreatedAt  int64
	TotalSize  uint64
	Files      []SnapshotFileInfo
	Version    uint32
	Checksum   string
}

// FileChunk represents a chunk of file data
type FileChunk struct {
	FileName    string
	Offset      uint64
	Data        []byte
	IsLastChunk bool
}

// SnapshotChunk represents a chunk of snapshot data
type SnapshotChunk struct {
	Metadata  *SnapshotMetadata
	FileChunk *FileChunk
	Sequence  uint64
	IsFinal   bool
}

// ImportSnapshotResponse represents the response from importing a snapshot
type ImportSnapshotResponse struct {
	Success       bool
	ErrorMessage  string
	SnapshotID    string
	BytesRestored uint64
	FilesRestored uint32
}

// SnapshotChunkBuilder builds a SnapshotChunk for import
type SnapshotChunkBuilder struct {
	chunk SnapshotChunk
}

// NewSnapshotChunkBuilder creates a new builder
func NewSnapshotChunkBuilder() *SnapshotChunkBuilder {
	return &SnapshotChunkBuilder{}
}

// WithMetadata sets the metadata (for first chunk)
func (b *SnapshotChunkBuilder) WithMetadata(meta *SnapshotMetadata) *SnapshotChunkBuilder {
	b.chunk.Metadata = meta
	return b
}

// WithFileChunk sets the file chunk data
func (b *SnapshotChunkBuilder) WithFileChunk(chunk *FileChunk) *SnapshotChunkBuilder {
	b.chunk.FileChunk = chunk
	return b
}

// Sequence sets the sequence number
func (b *SnapshotChunkBuilder) Sequence(seq uint64) *SnapshotChunkBuilder {
	b.chunk.Sequence = seq
	return b
}

// IsFinal marks this as the final chunk
func (b *SnapshotChunkBuilder) IsFinal(final bool) *SnapshotChunkBuilder {
	b.chunk.IsFinal = final
	return b
}

// Build returns the built chunk
func (b *SnapshotChunkBuilder) Build() (*SnapshotChunk, error) {
	// First chunk must have metadata
	if b.chunk.Sequence == 0 && b.chunk.Metadata == nil {
		return nil, errors.New("first chunk must have metadata")
	}
	return &b.chunk, nil
}

// SnapshotCollector collects snapshot chunks from export stream
type SnapshotCollector struct {
	Metadata *SnapshotMetadata
	Chunks   []SnapshotChunk
}

// NewSnapshotCollector creates a new collector
func NewSnapshotCollector() *SnapshotCollector {
	return &SnapshotCollector{
		Chunks: make([]SnapshotChunk, 0),
	}
}

// AddChunk adds a chunk to the collector
func (c *SnapshotCollector) AddChunk(chunk *SnapshotChunk) {
	if chunk.Metadata != nil {
		c.Metadata = chunk.Metadata
	}
	c.Chunks = append(c.Chunks, *chunk)
}

// GetTotalBytes returns the total bytes in the snapshot
func (c *SnapshotCollector) GetTotalBytes() uint64 {
	var total uint64
	for _, chunk := range c.Chunks {
		if chunk.FileChunk != nil {
			total += uint64(len(chunk.FileChunk.Data))
		}
	}
	return total
}

// IsComplete returns true if the snapshot is complete
func (c *SnapshotCollector) IsComplete() bool {
	if len(c.Chunks) == 0 {
		return false
	}
	lastChunk := c.Chunks[len(c.Chunks)-1]
	return lastChunk.IsFinal
}

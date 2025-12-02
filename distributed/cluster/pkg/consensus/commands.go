package consensus

import "encoding/json"

// NodeInfo holds minimal details for a node
type NodeInfo struct {
	ID      string
	Addr    string
	Cluster string
}

type CommandType int

const (
	CmdCreateCollection CommandType = iota + 1
	CmdInsert
	CmdDelete
	CmdUpdate
	CmdDropCollection
)

type Command struct {
	Type    CommandType     `json:"type"`
	Payload json.RawMessage `json:"payload"`
}

// CreateCollectionPayload for collection creation
type CreateCollectionPayload struct {
	CollectionName     string `json:"collection_name"`
	VectorDimension    int32  `json:"vector_dimension"`
	Distance           string `json:"distance"`
	PayloadTableSchema string `json:"payload_table_schema"`
}

// InsertPayload for vector insertion
type InsertPayload struct {
	CollectionName     string    `json:"collection_name"`
	ID                 int64     `json:"id"`
	Vector             []float32 `json:"vector"`
	PayloadInsertQuery string    `json:"payload_insert_query"`
}

// ApplyResult is returned from FSM.Apply
type ApplyResult struct {
	Data  interface{} `json:"data,omitempty"`
	Error error       `json:"error,omitempty"`
}

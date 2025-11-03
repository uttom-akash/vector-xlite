package types

type KeyValue struct {
	Key   string
	Value string
}

type SearchResultItem struct {
	Rowid    int64
	Distance float32
	Payload  []KeyValue
}

type SearchResponse struct {
	Results []SearchResultItem
}

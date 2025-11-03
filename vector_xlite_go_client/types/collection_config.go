package types

import "errors"

type CollectionConfig struct {
	CollectionName     string
	Distance           DistanceFunction
	VectorDimension    uint32
	PayloadTableSchema string
	IndexFilePath      string
}

type CollectionConfigBuilder struct {
	cfg CollectionConfig
}

func NewCollectionConfigBuilder() *CollectionConfigBuilder {
	return &CollectionConfigBuilder{}
}

func (b *CollectionConfigBuilder) CollectionName(n string) *CollectionConfigBuilder {
	b.cfg.CollectionName = n
	return b
}

func (b *CollectionConfigBuilder) Distance(d DistanceFunction) *CollectionConfigBuilder {
	b.cfg.Distance = d
	return b
}

func (b *CollectionConfigBuilder) VectorDimension(v uint32) *CollectionConfigBuilder {
	b.cfg.VectorDimension = v
	return b
}

func (b *CollectionConfigBuilder) PayloadTableSchema(s string) *CollectionConfigBuilder {
	b.cfg.PayloadTableSchema = s
	return b
}

func (b *CollectionConfigBuilder) IndexFilePath(p string) *CollectionConfigBuilder {
	b.cfg.IndexFilePath = p
	return b
}

func (b *CollectionConfigBuilder) Build() (*CollectionConfig, error) {
	if b.cfg.CollectionName == "" {
		return nil, errors.New("collection_name required")
	}
	if b.cfg.VectorDimension == 0 {
		return nil, errors.New("vector_dimension must be > 0")
	}
	return &b.cfg, nil
}

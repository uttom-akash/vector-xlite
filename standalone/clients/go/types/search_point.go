package types

import "errors"

type SearchPoint struct {
	CollectionName     string
	Vector             []float32
	TopK               uint32
	PayloadSearchQuery string // optional SQL to fetch/filter payload
}

type SearchPointBuilder struct {
	s SearchPoint
}

func NewSearchPointBuilder() *SearchPointBuilder { return &SearchPointBuilder{} }

func (b *SearchPointBuilder) CollectionName(n string) *SearchPointBuilder {
	b.s.CollectionName = n
	return b
}
func (b *SearchPointBuilder) Vector(v []float32) *SearchPointBuilder {
	b.s.Vector = v
	return b
}
func (b *SearchPointBuilder) TopK(k uint32) *SearchPointBuilder {
	b.s.TopK = k
	return b
}
func (b *SearchPointBuilder) PayloadSearchQuery(q string) *SearchPointBuilder {
	b.s.PayloadSearchQuery = q
	return b
}
func (b *SearchPointBuilder) Build() (*SearchPoint, error) {
	if b.s.CollectionName == "" {
		return nil, errors.New("collection_name required")
	}
	if len(b.s.Vector) == 0 {
		return nil, errors.New("vector required")
	}
	if b.s.TopK == 0 {
		b.s.TopK = 10
	}
	return &b.s, nil
}

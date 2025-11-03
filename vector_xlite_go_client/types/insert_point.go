package types

import "errors"

type InsertPoint struct {
	CollectionName     string
	Id                 int64
	Vector             []float32
	PayloadInsertQuery string // SQL using ?1 placeholder for rowid
}

type InsertPointBuilder struct {
	p InsertPoint
}

func NewInsertPointBuilder() *InsertPointBuilder { return &InsertPointBuilder{} }

func (b *InsertPointBuilder) CollectionName(n string) *InsertPointBuilder {
	b.p.CollectionName = n
	return b
}
func (b *InsertPointBuilder) Id(id int64) *InsertPointBuilder {
	b.p.Id = id
	return b
}
func (b *InsertPointBuilder) Vector(v []float32) *InsertPointBuilder {
	b.p.Vector = v
	return b
}
func (b *InsertPointBuilder) PayloadInsertQuery(q string) *InsertPointBuilder {
	b.p.PayloadInsertQuery = q
	return b
}
func (b *InsertPointBuilder) Build() (*InsertPoint, error) {
	if b.p.CollectionName == "" {
		return nil, errors.New("collection_name required")
	}
	if len(b.p.Vector) == 0 {
		return nil, errors.New("vector required")
	}
	return &b.p, nil
}

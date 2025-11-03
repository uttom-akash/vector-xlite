package types

type DistanceFunction int

const (
	DistanceUnknown DistanceFunction = iota
	DistanceCosine
	DistanceEuclidean
	DistanceDot
)

func (d DistanceFunction) String() string {
	switch d {
	case DistanceCosine:
		return "Cosine"
	case DistanceEuclidean:
		return "Euclidean"
	case DistanceDot:
		return "Dot"
	default:
		return "Unknown"
	}
}

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
		return "L2"
	case DistanceDot:
		return "IP"
	default:
		return "Unknown"
	}
}

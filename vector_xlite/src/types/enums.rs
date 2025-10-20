pub enum DistanceFunction {
    L2,
    Cosine,
    IP,
}

impl DistanceFunction {
    pub fn as_str(&self) -> &'static str {
        match self {
            DistanceFunction::L2 => "l2",
            DistanceFunction::Cosine => "cosine",
            DistanceFunction::IP => "ip",
        }
    }
}

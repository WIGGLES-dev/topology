use crate::coord::Coordinate;

pub trait Flavor {
    type Vertex: Coordinate;
    type Edge;
    type Face;
}

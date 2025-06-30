use crate::flavor;

pub trait Flavor {
    type Vertex;
    type Edge;
    type Face;
}

impl<Vertex, Edge, Face, F> flavor::Flavor for F
where
    F: Flavor<Vertex = Vertex, Edge = Edge, Face = Face>,
{
    const DIRECTED: flavor::Direction = flavor::Direction::Directed;
    const MULTIGRAPH: bool = true;
    const CYCLIC: bool = true;
    type Node = Vertex;
    type Edge = Edge;
}

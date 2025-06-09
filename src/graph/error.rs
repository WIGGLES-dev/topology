#[derive(Debug)]
pub enum Error {
    EdgeDoesNotExist,
    VertexDoesNotExist,
    CyclicReferenceInNonCyclicGraph,
    MultipleEdgesInNonMultiGraph,
    DirectedEdgeInUndirectedGraph,
    UnidrectedEdgeInDirectedGraph,
}

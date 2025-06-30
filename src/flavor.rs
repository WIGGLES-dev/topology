pub enum Direction {
    Directed,
    Undirected,
    Mixed,
}

pub trait Flavor {
    type Node;
    type Edge;
    const MULTIGRAPH: bool;
    const CYCLIC: bool;
    const DIRECTED: Direction;
}

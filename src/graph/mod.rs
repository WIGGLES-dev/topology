mod error;
mod traverser;
use std::marker::PhantomData;

pub use error::Error;

use crate::arena::{Arena, Key};

pub trait BaseNode {}
pub trait DirectedNode {}
pub trait BaseEdge {}
pub trait TwinEdge {}

pub struct Node<T> {
    pub weight: T,
    pub incoming: Option<EdgeKey>,
    pub outgoing: Option<EdgeKey>,
}

pub struct Edge<T> {
    pub weight: T,
    pub from: Key<NodeKey>,
    pub to: Key<NodeKey>,
    pub next: Key<EdgeKey>,
    pub twin: Option<Key<EdgeKey>>,
}

pub struct NodeKey;
pub struct EdgeKey;

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

pub struct Graph<F>
where
    F: Flavor,
{
    nodes: Arena<Node<F::Node>, NodeKey>,
    edges: Arena<Edge<F::Edge>, EdgeKey>,
    phantom: PhantomData<F>,
}

impl<F> Graph<F>
where
    F: Flavor,
{
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
            phantom: PhantomData,
        }
    }

    pub fn node(&self, key: Key<NodeKey>) -> Option<&F::Node> {
        match self.nodes.get(key) {
            Some(v) => Some(&v.weight),
            None => None,
        }
    }

    pub fn node_mut(&mut self, key: Key<NodeKey>) -> Option<&mut F::Node> {
        match self.nodes.get_mut(key) {
            Some(v) => Some(&mut v.weight),
            None => None,
        }
    }

    pub fn edge(&self, key: Key<EdgeKey>) -> Option<&F::Edge> {
        match self.edges.get(key) {
            Some(v) => Some(&v.weight),
            None => None,
        }
    }

    pub fn nodes(&self) -> &Arena<Node<F::Node>, NodeKey> {
        &self.nodes
    }

    pub fn edges(&self) -> &Arena<Edge<F::Edge>, EdgeKey> {
        &self.edges
    }

    pub fn insert_node(&mut self, weight: F::Node) -> Key<NodeKey> {
        self.nodes.insert(Node {
            weight,
            outgoing: None,
            incoming: None,
        })
    }

    pub fn insert_edge(
        &mut self,
        n1: Key<NodeKey>,
        n2: Key<NodeKey>,
        weight: F::Edge,
    ) -> Result<Key<EdgeKey>, Error> {
        todo!()
    }
}

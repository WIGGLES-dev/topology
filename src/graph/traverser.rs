use std::marker::PhantomData;

use crate::arena::Key;

use super::{EdgeKey, Flavor, Graph, error::Error};

pub struct Traverser<F>
where
    F: Flavor,
{
    start: Key<EdgeKey>,
    current: Key<EdgeKey>,
    phantom: PhantomData<F>,
}

impl<F> Traverser<F>
where
    F: Flavor,
{
    pub fn at(graph: &Graph<F>, edge: Key<EdgeKey>) -> Result<Self, Error> {
        graph.edges().get(edge).ok_or(Error::EdgeDoesNotExist)?;
        Ok(Self {
            start: edge,
            current: edge,
            phantom: PhantomData,
        })
    }

    pub fn next_local(&mut self, graph: &Graph<F>) {
        let next = graph.edges()[self.current].next;
        self.current = next;
    }
}

pub struct Descendants<'a, F>
where
    F: Flavor,
{
    graph: &'a Graph<F>,
    traverser: Traverser<F>,
}

pub struct Ancestors<'a, F>
where
    F: Flavor,
{
    graph: &'a Graph<F>,
    traverser: Traverser<F>,
}

use std::marker::PhantomData;

use super::{Dcel, EdgeKey, Key, VertexKey, error::Error};
use crate::{
    coord::{Coordinate, Orientation},
    util::ShoeString,
};

pub struct Traverser<V, E, F> {
    start: Key<EdgeKey>,
    edge: Key<EdgeKey>,
    phantom: PhantomData<(V, E, F)>,
}

impl<V, E, F> Clone for Traverser<V, E, F>
where
    F: Default,
{
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            edge: self.edge,
            phantom: PhantomData,
        }
    }
}

impl<V, E, F> Traverser<V, E, F>
where
    F: Default,
{
    pub fn new(dcel: &Dcel<V, E, F>, edge: Key<EdgeKey>) -> Result<Self, Error> {
        dcel.edges.get(edge).ok_or(Error::EdgeDoesNotExist)?;
        Ok(Self {
            start: edge,
            edge,
            phantom: PhantomData,
        })
    }

    pub fn at(dcel: &Dcel<V, E, F>, vertex: Key<VertexKey>) -> Result<Self, Error> {
        let edge = dcel
            .vertices
            .get(vertex)
            .ok_or(Error::VertexDoesNotExist)?
            .edge
            .ok_or(Error::IsolatedVertex)?;
        Ok(Self {
            start: edge,
            edge,
            phantom: PhantomData,
        })
    }

    pub fn start(&self) -> Key<EdgeKey> {
        self.start
    }

    pub fn edge(&self) -> Key<EdgeKey> {
        self.edge
    }

    pub fn is_line_segment(&mut self, dcel: &Dcel<V, E, F>) -> bool {
        let mut iter = TraverseThrough::new(dcel, self.edge).unwrap();
        while let Some(edge) = iter.next() {
            let next = dcel.edges[edge].next;
            let twin = dcel.edges[edge].twin;
            if next == twin {
                return true;
            }
        }
        false
    }

    pub fn is_at_start(&self) -> bool {
        self.start == self.edge
    }

    pub fn next(&mut self, dcel: &Dcel<V, E, F>) {
        let next = dcel.edges[self.edge].next;
        self.edge = next;
    }

    pub fn prev(&mut self, dcel: &Dcel<V, E, F>) {
        let prev = dcel.edges[self.edge].prev;
        self.edge = prev;
    }

    pub fn twin(&mut self, dcel: &Dcel<V, E, F>) {
        let twin = dcel.edges[self.edge].twin;
        self.edge = twin;
    }

    pub fn local_prev(&mut self, dcel: &Dcel<V, E, F>) {
        self.prev(dcel);
        self.twin(dcel);
    }

    pub fn local_next(&mut self, dcel: &Dcel<V, E, F>) {
        self.twin(dcel);
        self.next(dcel);
    }

    pub fn local_prev_next(
        dcel: &Dcel<V, E, F>,
        edge: Key<EdgeKey>,
    ) -> Result<(Key<EdgeKey>, Key<EdgeKey>), Error> {
        let mut traverser = Self::new(dcel, edge)?;
        traverser.local_prev(dcel);
        let prev = traverser.edge();
        traverser.reset();
        traverser.local_next(dcel);
        let next = traverser.edge();
        Ok((prev, next))
    }

    pub fn through<'a>(
        dcel: &'a Dcel<V, E, F>,
        edge: Key<EdgeKey>,
    ) -> Result<TraverseThrough<'a, V, E, F>, Error> {
        TraverseThrough::new(dcel, edge)
    }

    pub fn signed_area(dcel: &Dcel<V, E, F>, edge: Key<EdgeKey>) -> Result<f32, Error>
    where
        V: Coordinate,
    {
        let mut calc = ShoeString::default();
        for edge in Traverser::through(dcel, edge)? {
            let origin = dcel.edges[edge].origin;
            let twin_origin = dcel.edges[dcel.edges[edge].twin].origin;
            let v1 = &dcel.vertices[origin].weight;
            let v2 = &dcel.vertices[twin_origin].weight;
            calc.add(v1, v2);
        }
        Ok(calc.area())
    }

    pub fn around<'a>(
        dcel: &'a Dcel<V, E, F>,
        vertex: Key<VertexKey>,
    ) -> Result<TraverseAround<'a, V, E, F>, Error> {
        TraverseAround::new(dcel, vertex)
    }

    pub fn reset(&mut self) {
        self.edge = self.start;
    }
}

pub struct TraverseThrough<'a, V, E, F>
where
    F: Default,
{
    edge: Key<EdgeKey>,
    finished: bool,
    dcel: &'a Dcel<V, E, F>,
    traverser: Traverser<V, E, F>,
}

impl<'a, V, E, F> TraverseThrough<'a, V, E, F>
where
    F: Default,
{
    fn new(dcel: &'a Dcel<V, E, F>, edge: Key<EdgeKey>) -> Result<Self, Error> {
        let traverser = Traverser::new(dcel, edge)?;
        Ok(Self {
            edge,
            finished: false,
            dcel,
            traverser,
        })
    }
}

impl<'a, V, E, F> Iterator for TraverseThrough<'a, V, E, F>
where
    F: Default,
{
    type Item = Key<EdgeKey>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let next = self.traverser.edge;
        self.traverser.next(&self.dcel);

        if self.traverser.start == self.traverser.edge {
            self.finished = true;
        }

        Some(next)
    }
}

pub struct TraverseAround<'a, V, E, F>
where
    F: Default,
{
    vertex: Key<VertexKey>,
    finished: bool,
    dcel: &'a Dcel<V, E, F>,
    traverser: Traverser<V, E, F>,
}

impl<'a, V, E, F> TraverseAround<'a, V, E, F>
where
    F: Default,
{
    pub fn new(dcel: &'a Dcel<V, E, F>, vertex: Key<VertexKey>) -> Result<Self, Error> {
        let edge = dcel
            .vertices
            .get(vertex)
            .ok_or(Error::VertexDoesNotExist)?
            .edge
            .ok_or(Error::IsolatedVertex)?;
        let traverser = Traverser::new(dcel, edge)?;
        Ok(Self {
            vertex,
            finished: false,
            dcel,
            traverser,
        })
    }
}

impl<'a, V, E, F> Iterator for TraverseAround<'a, V, E, F>
where
    F: Default,
{
    type Item = Key<EdgeKey>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let next = self.traverser.edge;

        self.traverser.local_next(&self.dcel);

        if self.traverser.start == self.traverser.edge {
            self.finished = true;
        }

        Some(next)
    }
}

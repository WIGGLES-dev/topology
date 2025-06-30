use std::marker::PhantomData;

use super::{Dcel, EdgeKey, Key, VertexKey, error::Error};
use crate::{
    coord::{Coordinate, Orientation},
    dcel::{FaceKey, FaceMask, flavor::Flavor, traverser},
    util::ShoeString,
};

pub struct Traverser<F: Flavor> {
    start: Key<EdgeKey>,
    edge: Key<EdgeKey>,
    phantom: PhantomData<(F::Vertex, F::Edge, F::Face)>,
}

impl<F: Flavor> Clone for Traverser<F> {
    fn clone(&self) -> Self {
        Self {
            start: self.start,
            edge: self.edge,
            phantom: PhantomData,
        }
    }
}

impl<F: Flavor> Traverser<F> {
    pub fn new(dcel: &Dcel<F>, edge: Key<EdgeKey>) -> Result<Self, Error> {
        dcel.edges.get(edge).ok_or(Error::EdgeDoesNotExist)?;
        Ok(Self {
            start: edge,
            edge,
            phantom: PhantomData,
        })
    }

    pub fn at(dcel: &Dcel<F>, vertex: Key<VertexKey>) -> Result<Self, Error> {
        let edge = dcel
            .vertices
            .get(vertex)
            .ok_or(Error::VertexDoesNotExist)?
            .edge
            .ok_or(Error::DisconnectedVertex)?;
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

    pub fn is_line_segment(&mut self, dcel: &Dcel<F>) -> bool {
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

    pub fn next(&mut self, dcel: &Dcel<F>) {
        let next = dcel.edges[self.edge].next;
        self.edge = next;
    }

    pub fn prev(&mut self, dcel: &Dcel<F>) {
        let prev = dcel.edges[self.edge].prev;
        self.edge = prev;
    }

    pub fn twin(&mut self, dcel: &Dcel<F>) {
        let twin = dcel.edges[self.edge].twin;
        self.edge = twin;
    }

    pub fn local_prev(&mut self, dcel: &Dcel<F>) {
        self.prev(dcel);
        self.twin(dcel);
    }

    pub fn local_next(&mut self, dcel: &Dcel<F>) {
        self.twin(dcel);
        self.next(dcel);
    }

    pub fn local_prev_next(dcel: &Dcel<F>, edge: Key<EdgeKey>) -> Result<[Key<EdgeKey>; 2], Error> {
        let mut traverser = Self::new(dcel, edge)?;
        traverser.local_prev(dcel);
        let prev = traverser.edge();
        traverser.reset();
        traverser.local_next(dcel);
        let next = traverser.edge();
        Ok([prev, next])
    }

    pub fn through<'a>(
        dcel: &'a Dcel<F>,
        edge: Key<EdgeKey>,
    ) -> Result<TraverseThrough<'a, F>, Error> {
        TraverseThrough::new(dcel, edge)
    }

    pub fn signed_area(dcel: &Dcel<F>, edge: Key<EdgeKey>) -> Result<f32, Error>
    where
        F::Vertex: Coordinate,
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
        dcel: &'a Dcel<F>,
        vertex: Key<VertexKey>,
    ) -> Result<TraverseAround<'a, F>, Error> {
        TraverseAround::new(dcel, vertex)
    }

    pub fn reset(&mut self) {
        self.edge = self.start;
    }
}

pub struct TraverseThrough<'a, F: Flavor> {
    edge: Key<EdgeKey>,
    finished: bool,
    dcel: &'a Dcel<F>,
    traverser: Traverser<F>,
}

impl<'a, F: Flavor> TraverseThrough<'a, F> {
    fn new(dcel: &'a Dcel<F>, edge: Key<EdgeKey>) -> Result<Self, Error> {
        let traverser = Traverser::new(dcel, edge)?;
        Ok(Self {
            edge,
            finished: false,
            dcel,
            traverser,
        })
    }
}

impl<'a, F: Flavor> Iterator for TraverseThrough<'a, F> {
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

pub struct TraverseAround<'a, F: Flavor> {
    vertex: Key<VertexKey>,
    finished: bool,
    dcel: &'a Dcel<F>,
    traverser: Traverser<F>,
}

impl<'a, F: Flavor> TraverseAround<'a, F> {
    pub fn new(dcel: &'a Dcel<F>, vertex: Key<VertexKey>) -> Result<Self, Error> {
        let edge = dcel
            .vertices
            .get(vertex)
            .ok_or(Error::VertexDoesNotExist)?
            .edge
            .ok_or(Error::DisconnectedVertex)?;
        let traverser = Traverser::new(dcel, edge)?;
        Ok(Self {
            vertex,
            finished: false,
            dcel,
            traverser,
        })
    }
}

impl<'a, F: Flavor> Iterator for TraverseAround<'a, F> {
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

impl<'a, F: Flavor> DoubleEndedIterator for TraverseAround<'a, F> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.traverser.is_at_start() {
            return None;
        }

        self.traverser.local_prev(&self.dcel);

        Some(self.traverser.edge)
    }
}

/// Iterator that follows a path of edges to the outside
pub struct TraverseOutwards<'a, F: Flavor> {
    edge: Key<EdgeKey>,
    finished: bool,
    dcel: &'a Dcel<F>,
    traverser: Traverser<F>,
}

impl<'a, F: Flavor> Iterator for TraverseOutwards<'a, F> {
    type Item = Key<EdgeKey>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        self.traverser.next(&self.dcel);
        self.traverser.twin(&self.dcel);

        let edge = self.dcel.edge(self.traverser.edge());

        let face = self.dcel.face(edge.face);
        if face.mask.contains(FaceMask::IS_OUTER) {
            self.finished = true;
        };

        Some(self.traverser.edge())
    }
}

impl<'a, F: Flavor> TraverseOutwards<'a, F> {
    pub fn root(&mut self) -> Key<FaceKey> {
        let last_edge_key = self.last().unwrap_or(self.edge);
        let edge = self.dcel.edge(last_edge_key);
        edge.face.clone()
    }
}

pub struct TraverseSiblingFaces<'a, F: Flavor> {
    face: Key<FaceKey>,
    dcel: &'a Dcel<F>,
    traverser: Traverser<F>,
}

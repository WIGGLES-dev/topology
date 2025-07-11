use std::convert::Infallible;

use crate::{
    arena::Key,
    coord::Coordinate,
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, FaceKey, FacePtrs, Keyed, Traverser, Vertex, VertexKey,
        VertexPtrs,
        flavor::Flavor,
        linker::Linker,
        ops::{Operator, OperatorErr},
    },
};

/// Make a Vertex + Edge connected to an existing Vertex
pub struct Mve<F: Flavor> {
    pub origin: Key<VertexKey>,
    pub edges: [F::Edge; 2],
    pub vertex: F::Vertex,
    /// two outgoing edges that define the range of edges that need to move to the new vertex
    pub(crate) reparent: Vec<Key<EdgeKey>>,
}

impl<F: Flavor> Mve<F> {
    pub fn new(origin: Key<VertexKey>, edges: [F::Edge; 2], vertex: F::Vertex) -> Self {
        Self {
            vertex,
            edges,
            origin,
            reparent: vec![],
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MevError {
    #[error("local cycle does not have uniform faces")]
    NextPrevFaceMismatch,
}

impl<F: Flavor> Operator<F> for Mve<F>
where
    F::Vertex: Coordinate,
{
    type Inverse = Kve;
    type Error = MevError;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        //
        //    > o >
        //  a |   | b
        //   <  o  <
        //

        //    > o >
        //  c |   | d
        //      o
        //  a |   | b
        //   <  o  <

        //
        //    > o >
        //  c |   |d
        //    |   |  e
        //    |   >_____
        //    | o  _____ 0
        //    |   <
        //    |   | f
        //    |   |
        //  a |   | b
        //   <  o  <

        let [he1_weight, he2_weight] = self.edges;
        let vertex_weight = self.vertex;

        let outgoing = dcel.edges.reserve();
        let incoming = dcel.edges.reserve();

        let vertex = dcel.vertices.insert(Vertex {
            inner: VertexPtrs {
                edge: Some(incoming),
            },
            weight: vertex_weight,
        });

        let [outgoing_prev, outgoing_next] = Linker::find_prev_next(dcel, self.origin, vertex);
        let outgoing_face = outgoing_prev.twin(dcel).face(dcel);

        dcel.edges.set(
            outgoing,
            Edge {
                inner: EdgePtrs {
                    origin: self.origin,
                    twin: incoming,
                    prev: outgoing_prev,
                    next: incoming,
                    face: outgoing_face,
                },
                weight: he1_weight,
            },
        );

        dcel.edges.set(
            incoming,
            Edge {
                inner: EdgePtrs {
                    origin: vertex,
                    twin: outgoing,
                    prev: outgoing,
                    next: outgoing,
                    face: outgoing_face,
                },
                weight: he2_weight,
            },
        );

        Linker::splice_edge(dcel, outgoing, outgoing_prev, outgoing_next);
        Linker::splice_edge(dcel, incoming, incoming, incoming);

        // carry over some edges from the origin when making the new vertex. they wil now go to
        if self.reparent.len() > 0 {
            let mut linker = Linker::new();
            linker.reparent_vertex(dcel, vertex, self.origin, Some(self.reparent));
        }

        Ok(Kve {
            origin: self.origin,
            vertex,
            edges: [outgoing, incoming],
        })
    }
}

pub struct Kve {
    pub origin: Key<VertexKey>,
    pub vertex: Key<VertexKey>,
    pub edges: [Key<EdgeKey>; 2],
}

impl Kve {
    pub fn new(origin: Key<VertexKey>, vertex: Key<VertexKey>, edges: [Key<EdgeKey>; 2]) -> Self {
        Self {
            origin,
            vertex,
            edges,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum KveError {
    #[error("this operationw would kill a face")]
    WouldKillFace,
    #[error("neither edge has the target vertex as an origin")]
    EdgeVertexMismatch,
}

impl<F: Flavor> Operator<F> for Kve
where
    F::Vertex: Coordinate,
{
    type Inverse = Mve<F>;
    type Error = KveError;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        let [outgoing, incoming] = self.edges;
        let outgoing_face = outgoing.face(dcel);
        let incoming_face = incoming.face(dcel);

        let outgoing_count = Traverser::through(dcel, outgoing_face.edge(dcel))
            .unwrap()
            .count();
        let incoming_count = Traverser::through(dcel, incoming_face.edge(dcel))
            .unwrap()
            .count();

        if outgoing_count == 3 || incoming_count == 3 {
            return Err(KveError::WouldKillFace);
        }

        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let mut linker = Linker::new();
        // dcel.vertex(self.vertex);

        let [outgoing, incoming] = self.edges;

        // unsplice edges from the graph
        Linker::unsplice_edge(dcel, self.edges);
        let outgoing = dcel.edges.remove(outgoing).unwrap();
        let incoming = dcel.edges.remove(incoming).unwrap();

        // reparent remaining edge around vertex
        let reparent = linker.reparent_vertex(dcel, self.origin, self.vertex, None);

        // remove vertex
        let v = dcel.vertices.remove(self.vertex).unwrap();

        Ok(Mve {
            origin: self.origin,
            vertex: v.weight,
            edges: [outgoing, incoming].map(|v| v.weight),
            reparent,
        })
    }
}

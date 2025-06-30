use std::convert::Infallible;

use crate::{
    arena::Key,
    coord::Coordinate,
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, Face, FaceKey, FaceMask, FacePtrs, Keyed, Traverser, Vertex,
        VertexKey, VertexPtrs,
        flavor::Flavor,
        linker::Linker,
        ops::{Operator, OperatorErr},
    },
};

/// Make a Vertex + Edge connected to an existing Vertex
pub struct Mef<F: Flavor> {
    pub vertices: [Key<VertexKey>; 2],
    pub data: (F::Face, F::Edge, F::Edge),
}

impl<F: Flavor> Mef<F> {
    pub fn new(vertices: [Key<VertexKey>; 2], data: (F::Face, F::Edge, F::Edge)) -> Self {
        Self { vertices, data }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MefError {
    #[error("face does not match")]
    FaceMismatch,
    #[error("isolated vertex")]
    IsolatedVertex,
}

impl<F: Flavor> Operator<F> for Mef<F>
where
    F::Vertex: Coordinate,
{
    type Inverse = Kef;
    type Error = MefError;
    type Check = Key<FaceKey>;

    fn check(&self, dcel: &Dcel<F>) -> Result<Self::Check, Self::Error> {
        let [v1, v2] = self.vertices;
        dcel.vertices.get(v1).ok_or(MefError::IsolatedVertex)?;
        dcel.vertices.get(v2).ok_or(MefError::IsolatedVertex)?;

        for edge in Traverser::around(dcel, v1).unwrap() {
            for edge in Traverser::through(dcel, edge).unwrap() {
                if edge.twin(dcel).origin(dcel) == v2 {
                    return Ok(edge.face(dcel));
                }
            }
        }
        return Err(MefError::FaceMismatch);
    }

    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut Dcel<F>,
    ) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        // before:
        //     >               >
        //   a0 \             / a2
        //      v1          v2
        //   b0 /             \ b2
        //     <               <
        //
        // after:
        //     >               >
        //   a0 \    outgoing ->    / a2
        //       v1         v2
        //   b0 /    <- b1    \ b2
        //     <               <
        //

        let [v1, v2] = self.vertices;

        let outgoing = dcel.edges.reserve();
        let incoming = dcel.edges.reserve();

        let [outgoing_local_prev, outgoing_local_next] = Linker::find_prev_next(dcel, v1, v2);
        let [incoming_local_prev, incoming_local_next] = Linker::find_prev_next(dcel, v2, v1);

        // incoming is set to be the opposite of mvvef so that edge loops alternate direction
        let face = dcel.faces.insert(Face {
            inner: FacePtrs {
                edge: incoming,
                holes: vec![],
                mask: FaceMask::IS_BOUNDARY,
            },
            weight: self.data.0,
        });

        dcel.edges.set(
            outgoing,
            Edge {
                inner: EdgePtrs {
                    origin: v1,
                    twin: incoming,
                    prev: incoming,
                    next: incoming,
                    face: outgoing_local_prev.twin(dcel).face(dcel),
                },
                weight: self.data.1,
            },
        );

        dcel.edges.set(
            incoming,
            Edge {
                inner: EdgePtrs {
                    origin: v2,
                    twin: outgoing,
                    prev: outgoing,
                    next: outgoing,
                    face: face,
                },
                weight: self.data.2,
            },
        );

        Linker::splice_edge(dcel, outgoing, outgoing_local_prev, outgoing_local_next);
        Linker::splice_edge(dcel, incoming, incoming_local_prev, incoming_local_next);

        dcel.propagate_face(incoming, face).unwrap();

        Ok(Kef {
            face,
            edges: [outgoing, incoming],
        })
    }
}

pub struct Kef {
    pub face: Key<FaceKey>,
    pub edges: [Key<EdgeKey>; 2],
}

impl Kef {
    pub fn new(face: Key<FaceKey>, edges: [Key<EdgeKey>; 2]) -> Self {
        Self { face, edges }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum KefError {}

impl<F: Flavor> Operator<F> for Kef
where
    F::Vertex: Coordinate,
{
    type Inverse = Mef<F>;
    type Error = Infallible;
    type Check = ();

    fn check(&self, dcel: &Dcel<F>) -> Result<Self::Check, Self::Error> {
        Ok(())
    }
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut Dcel<F>,
    ) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let [e1, e2] = self.edges;
        println!("kef {e1} {e2}");
        let e1f = e1.face(dcel);
        let e2f = e2.face(dcel);
        let rface = if self.face == e1f { e2f } else { e1f };

        dcel.propagate_face(e1, rface).unwrap();
        dcel.propagate_face(e2, rface).unwrap();

        Linker::unsplice_edge(dcel, self.edges);

        let outgoing = dcel.edges.remove(self.edges[0]).unwrap();
        let incoming = dcel.edges.remove(self.edges[1]).unwrap();
        let face = dcel.faces.remove(self.face).unwrap();

        Ok(Mef {
            vertices: [outgoing.origin, incoming.origin],
            data: (face.weight, outgoing.weight, incoming.weight),
        })
    }
}

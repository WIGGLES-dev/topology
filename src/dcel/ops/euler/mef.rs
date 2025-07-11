use std::convert::Infallible;

use crate::{
    arena::Key,
    coord::{Coordinate, Orientation},
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, Face, FaceKey, FaceMask, FacePtrs, Keyed, Op, Traverser,
        Vertex, VertexKey, VertexPtrs,
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

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        let [v1, v2] = self.vertices;
        let Some(_) = dcel.vertices.get(v1) else {
            return Err(MefError::IsolatedVertex);
        };
        let Some(_) = dcel.vertices.get(v2) else {
            return Err(MefError::IsolatedVertex);
        };

        let [outgoing_local_prev, outgoing_local_next] = Linker::find_prev_next(dcel, v1, v2);
        let [incoming_local_prev, incoming_local_next] = Linker::find_prev_next(dcel, v2, v1);

        // this will become outgoing.next
        let outgoing_face = incoming_local_prev.face(dcel);
        // this will become incoming.next
        let incoming_face = outgoing_local_prev.face(dcel);

        // if they aren't equal we can't split this face
        if outgoing_face != incoming_face {
            return Err(MefError::FaceMismatch);
        }

        Ok(())
    }

    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        // before:
        //     >               >
        //   a0 \             / a2
        //      v1          v2
        //   b0 /             \ b2
        //     <               <
        //
        // after:
        //     >               >
        //   a0 \    a1 ->    / a2
        //       v1         v2
        //   b0 /    <- b1    \ b2
        //     <               <
        //

        let [v1, v2] = self.vertices;

        let [outgoing_local_prev, outgoing_local_next] = Linker::find_prev_next(dcel, v1, v2);
        let [incoming_local_prev, incoming_local_next] = Linker::find_prev_next(dcel, v2, v1);

        // this will become outgoing.next
        let outgoing_face = incoming_local_prev.face(dcel);
        // this will become incoming.next
        let incoming_face = outgoing_local_prev.face(dcel);

        // if they aren't equal we can't split this face
        if outgoing_face != incoming_face {
            return Err(OperatorErr {
                op: self,
                err: MefError::FaceMismatch,
            });
        }
        let input = &outgoing_face;

        let outgoing = dcel.edges.reserve();
        let incoming = dcel.edges.reserve();

        dcel.edges.set(
            outgoing,
            Edge {
                inner: EdgePtrs {
                    origin: v1,
                    twin: incoming,
                    prev: incoming,
                    next: incoming,
                    face: *input,
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
                    face: *input,
                },
                weight: self.data.2,
            },
        );

        // Linker::splice_edge(dcel, outgoing, outgoing_local_prev, outgoing_local_next);
        // Linker::splice_edge(dcel, incoming, incoming_local_prev, incoming_local_next);

        Linker::follow(dcel, outgoing_local_next.twin(dcel), outgoing);
        Linker::follow(dcel, outgoing, incoming_local_prev);

        Linker::follow(dcel, incoming_local_next.twin(dcel), incoming);
        Linker::follow(dcel, incoming, outgoing_local_prev);

        let face_orientation = Traverser::shoestring(dcel, input.edge(dcel))
            .unwrap()
            .orientation();

        let outgoing_orientation = Traverser::shoestring(dcel, outgoing).unwrap().orientation();

        // use the edge that causes the opposite orientation of the face we are splitting
        let propagate = if face_orientation == outgoing_orientation {
            incoming
        } else {
            outgoing
        };
        let face = dcel.faces.insert(Face {
            inner: FacePtrs {
                edge: propagate,
                holes: vec![],
                mask: FaceMask::IS_BOUNDARY,
            },
            weight: self.data.0,
        });

        dcel.propagate_face(propagate, face).unwrap();

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

#[derive(thiserror::Error, Debug)]
pub enum KefError {
    #[error("splitting face and face do not share an edge")]
    FaceMismatch,
}

impl<F: Flavor> Operator<F> for Kef
where
    F::Vertex: Coordinate,
{
    type Inverse = Mef<F>;
    type Error = Infallible;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let [e1, e2] = self.edges;
        println!("kef {e1} {e2}");
        let e1f = e1.face(dcel);
        let e2f = e2.face(dcel);
        let rface = if self.face == e1f { e2f } else { e1f };

        dcel.propagate_face(e1, rface).unwrap();
        dcel.propagate_face(e2, rface).unwrap();

        Linker::unsplice_edge(dcel, self.edges);

        let outgoing = dcel.edges.remove(e1).unwrap();
        let incoming = dcel.edges.remove(e2).unwrap();
        let face = dcel.faces.remove(self.face).unwrap();

        Ok(Mef {
            vertices: [outgoing.origin, incoming.origin],
            data: (face.weight, outgoing.weight, incoming.weight),
        })
    }
}

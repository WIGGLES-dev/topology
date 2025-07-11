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

pub struct Mvvef<F: Flavor> {
    pub data: (F::Vertex, F::Vertex, F::Edge, F::Edge, F::Face),
}

impl<F: Flavor> Operator<F> for Mvvef<F> {
    type Error = Infallible;
    type Inverse = Kvvef;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let outgoing = dcel.edges.reserve();
        let incoming = dcel.edges.reserve();

        let v1 = dcel.vertices.insert(Vertex {
            inner: VertexPtrs {
                edge: Some(outgoing),
            },
            weight: self.data.0,
        });
        let v2 = dcel.vertices.insert(Vertex {
            inner: VertexPtrs {
                edge: Some(incoming),
            },
            weight: self.data.1,
        });

        let face = dcel.faces.insert(Face {
            inner: FacePtrs {
                edge: outgoing,
                holes: vec![],
                mask: FaceMask::default(),
            },
            weight: self.data.4,
        });

        dcel.edges.set(
            outgoing,
            Edge {
                inner: EdgePtrs {
                    origin: v1,
                    twin: incoming,
                    prev: incoming,
                    next: incoming,
                    face,
                },
                weight: self.data.2,
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
                    face,
                },
                weight: self.data.3,
            },
        );

        dcel.bounding_face.get_or_insert(face);

        Ok(Kvvef {
            edges: [outgoing, incoming],
            vertices: [v1, v2],
            face,
        })
    }
}

pub struct Kvvef {
    pub vertices: [Key<VertexKey>; 2],
    pub edges: [Key<EdgeKey>; 2],
    pub face: Key<FaceKey>,
}

impl<F: Flavor> Operator<F> for Kvvef {
    type Error = Infallible;
    type Inverse = Mvvef<F>;
    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let [v1, v2] = self.vertices;
        let [e1, e2] = self.edges;

        Ok(Mvvef {
            data: (
                dcel.vertices.remove(v1).unwrap().weight,
                dcel.vertices.remove(v2).unwrap().weight,
                dcel.edges.remove(e1).unwrap().weight,
                dcel.edges.remove(e2).unwrap().weight,
                dcel.faces.remove(self.face).unwrap().weight,
            ),
        })
    }
}

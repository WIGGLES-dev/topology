pub mod draw;
mod entities;
pub mod error;
mod flavor;
mod index;
mod linker;
pub mod ops;
#[cfg(test)]
mod tests;
mod traverser;
mod util;
pub mod vis;

use std::ops::{Deref, DerefMut};

use crate::{arena::ArenaBitMask, dcel::linker::Linker, util::ShoeString};

use error::Error::{self, EdgeDoesNotExist, FaceDoesNotExist, VertexDoesNotExist};

pub use entities::*;
pub use flavor::Flavor;
pub use ops::{Op, Operator, OperatorErr};
pub use traverser::*;

use crate::{
    arena::{Arena, Key},
    coord::{Coordinate, FromCoordinate, Precision, UpdateCoordinate, sort_clockwise},
};

pub struct Dcel<F: Flavor> {
    /// all vertices
    pub vertices: Arena<Vertex<F::Vertex>, VertexKey>,
    /// all half edge pairs
    pub edges: Arena<Edge<F::Edge>, EdgeKey>,
    /// all faces
    pub faces: Arena<Face<F::Face>, FaceKey>,
    /// the top level bounding face
    bounding_face: Option<Key<FaceKey>>,
}

impl<F: Flavor> Dcel<F> {
    pub fn vertices(&self) -> &Arena<Vertex<F::Vertex>, VertexKey> {
        &self.vertices
    }

    pub fn vertex(&self, key: Key<VertexKey>) -> &Vertex<F::Vertex> {
        &self.vertices[key]
    }

    pub fn vertex_mut(&mut self, key: Key<VertexKey>) -> &mut Vertex<F::Vertex> {
        &mut self.vertices[key]
    }

    pub fn edges(&self) -> &Arena<Edge<F::Edge>, EdgeKey> {
        &self.edges
    }

    pub fn edge(&self, key: Key<EdgeKey>) -> &Edge<F::Edge> {
        &self.edges[key]
    }

    pub fn edge_mut(&mut self, key: Key<EdgeKey>) -> &mut Edge<F::Edge> {
        &mut self.edges[key]
    }

    pub fn faces(&self) -> &Arena<Face<F::Face>, FaceKey> {
        &self.faces
    }

    pub fn face(&self, key: Key<FaceKey>) -> &Face<F::Face> {
        &self.faces[key]
    }

    pub fn face_mut(&mut self, key: Key<FaceKey>) -> &mut Face<F::Face> {
        &mut self.faces[key]
    }
}

impl<F: Flavor> Dcel<F>
where
    F::Vertex: Coordinate + FromCoordinate,
    F::Edge: Default,
    F::Face: Default,
{
}

impl<F: Flavor> Default for Dcel<F> {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            edges: Default::default(),
            faces: Default::default(),
            bounding_face: None,
        }
    }
}

impl<F: Flavor> Dcel<F> {
    pub fn from_raw(
        vertices: Arena<Vertex<F::Vertex>, VertexKey>,
        edges: Arena<Edge<F::Edge>, EdgeKey>,
        faces: Arena<Face<F::Face>, FaceKey>,
        bounding_face: Option<Key<FaceKey>>,
    ) -> Self {
        Self {
            vertices,
            edges,
            faces,
            bounding_face,
        }
    }

    pub fn through<Cb>(&mut self, edge: Key<EdgeKey>, mut cb: Cb) -> Result<(), Error>
    where
        Cb: FnMut(&mut Self, Key<EdgeKey>),
    {
        let mut traverser = Traverser::new(&self, edge)?;

        loop {
            cb(self, traverser.edge());
            traverser.next(&self);
            if traverser.is_at_start() {
                break;
            }
        }
        Ok(())
    }

    pub fn around<Cb>(&mut self, vertex: Key<VertexKey>, mut cb: Cb) -> Result<(), Error>
    where
        Cb: FnMut(&mut Self, Key<EdgeKey>),
    {
        let mut traverser = Traverser::at(&self, vertex)?;
        loop {
            let edge = traverser.edge();
            // super dangerous to mutate the dcel while you're traversing it.
            traverser.local_next(&self);
            cb(self, edge);
            if traverser.is_at_start() {
                break;
            }
        }
        Ok(())
    }

    pub fn face_signed_area(&self, key: Key<FaceKey>) -> Result<f32, Error>
    where
        F::Vertex: Coordinate,
    {
        let face = self.face(key);
        Traverser::signed_area(&self, face.edge)
    }

    pub fn face_path(&self, key: Key<FaceKey>) -> Result<Vec<f32>, Error>
    where
        F::Vertex: Coordinate,
    {
        let incident = self.faces.get(key).ok_or(FaceDoesNotExist)?.edge;
        let mut traverser = Traverser::new(self, incident)?;

        let mut path = vec![];
        path.extend(self.vertices[self.edges[incident].origin].weight.xy());

        loop {
            let origin = traverser.edge().origin(self).weight(self);
            path.extend(origin.xy());
            traverser.next(self);
            if traverser.is_at_start() {
                break;
            }
        }

        Ok(path)
    }

    pub(crate) fn propagate_face(
        &mut self,
        edge: Key<EdgeKey>,
        face: Key<FaceKey>,
    ) -> Result<(), Error> {
        let mut traverser = Traverser::new(&self, edge)?;

        loop {
            self.edge_mut(traverser.edge()).face = face;
            traverser.next(&self);

            if traverser.is_at_start() {
                break;
            }
        }

        Ok(())
    }

    pub fn check_apply<Op: Operator<F>>(
        &mut self,
        op: Op,
    ) -> Result<Op::Inverse, OperatorErr<Op, Op::Error>> {
        match op.check(self) {
            Ok(input) => op.apply(self),
            Err(err) => Err(OperatorErr { op, err }),
        }
    }
}

macro_rules! op_res {
    ($flavor:ty => $op:ty) => {
        Result<
            <$op as Operator<$flavor>>::Inverse,
            OperatorErr<
                $op,
                <$op as Operator<$flavor>>::Error
            >
        >
    };
}

impl<F: Flavor> Dcel<F>
where
    F::Edge: Default,
    F::Face: Default,
{
    pub fn mef(&mut self, from: Key<VertexKey>, to: Key<VertexKey>) -> op_res!(F => ops::Mef<F>)
    where
        F::Vertex: Coordinate,
    {
        self.check_apply(
            ops::Mef {
                vertices: [from, to],
                data: (Default::default(), Default::default(), Default::default()),
            }
            .into(),
        )
    }

    pub fn kef(&mut self, face: Key<FaceKey>, edges: [Key<EdgeKey>; 2]) -> op_res!(F => ops::Kef)
    where
        F::Vertex: Coordinate,
    {
        self.check_apply(ops::Kef { face, edges }.into())
    }

    pub fn mekh(&mut self) -> op_res!(F => ops::Mekh) {
        todo!()
    }

    pub fn kemh(&mut self) -> op_res!(F => ops::Kemh) {
        todo!()
    }

    pub fn mve(&mut self, origin: Key<VertexKey>, vw: F::Vertex) -> op_res!(F => ops::Mve<F>)
    where
        F::Vertex: Coordinate,
    {
        self.check_apply(ops::Mve {
            origin,
            vertex: vw,
            edges: [Default::default(), Default::default()],
            reparent: vec![],
        })
    }

    pub fn kve(
        &mut self,
        origin: Key<VertexKey>,
        edges: [Key<EdgeKey>; 2],
        vertex: Key<VertexKey>,
    ) -> op_res!(F => ops::Kve)
    where
        F::Vertex: Coordinate,
    {
        self.check_apply(ops::Kve {
            origin,
            edges,
            vertex,
        })
    }

    pub fn mvvef(&mut self, v1: F::Vertex, v2: F::Vertex) -> op_res!(F => ops::Mvvef<F>) {
        self.check_apply(ops::Mvvef {
            data: (
                v1,
                v2,
                Default::default(),
                Default::default(),
                Default::default(),
            ),
        })
    }

    pub fn mvh(&mut self, vertex: F::Vertex) -> op_res!(F => ops::Mvh<F>) {
        self.check_apply(ops::Mvh { vertex })
    }

    pub fn kvh(&mut self, vertex: Key<VertexKey>) -> op_res!(F => ops::Kvh) {
        self.check_apply(ops::Kvh { vertex })
    }
}

impl<F: Flavor> Dcel<F> {
    pub fn translate_vertex(
        &mut self,
        key: Key<VertexKey>,
        delta: impl Coordinate,
    ) -> op_res!(F => ops::TranslateVertex)
    where
        F::Vertex: Coordinate + UpdateCoordinate,
    {
        self.check_apply(ops::TranslateVertex {
            vertex: key,
            delta: delta.xyz(),
        })
    }

    pub fn translate_vertex_abs(
        &mut self,
        key: Key<VertexKey>,
        coord: impl Coordinate,
    ) -> op_res!(F => ops::TranslateVertex)
    where
        F::Vertex: Coordinate + UpdateCoordinate,
    {
        self.check_apply(ops::TranslateVertex::from_absolute(self, key, coord))
    }
}

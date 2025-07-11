use std::convert::Infallible;

use crate::{
    arena::Key,
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, Face, FaceKey, FaceMask, FacePtrs, Vertex, VertexKey,
        VertexPtrs,
        flavor::Flavor,
        ops::{Operator, OperatorErr},
    },
};

pub struct Mvh<F: Flavor> {
    pub vertex: F::Vertex,
}

#[derive(thiserror::Error, Debug)]
pub enum MvhError {}

impl<F: Flavor> Operator<F> for Mvh<F> {
    type Error = Infallible;
    type Inverse = Kvh;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let vertex = dcel.vertices.insert(Vertex {
            inner: VertexPtrs { edge: None },
            weight: self.vertex,
        });

        Ok(Kvh { vertex })
    }
}

pub struct Kvh {
    pub vertex: Key<VertexKey>,
}

#[derive(thiserror::Error, Debug)]
pub enum KvError {
    #[error("vertex does not exist")]
    VertexDoesNotExist,
}

impl<F: Flavor> Operator<F> for Kvh {
    type Error = KvError;
    type Inverse = Mvh<F>;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let vertex = dcel.vertices.remove(self.vertex).unwrap().weight;
        match dcel.vertices.remove(self.vertex) {
            Some(Vertex { weight, .. }) => Ok(Mvh { vertex }),
            None => Err(OperatorErr {
                op: self,
                err: KvError::VertexDoesNotExist,
            }),
        }
    }
}

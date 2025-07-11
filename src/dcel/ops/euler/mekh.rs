use crate::{
    arena::Key,
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, FaceKey, FacePtrs, Keyed, Traverser, Vertex, VertexKey,
        VertexPtrs,
        flavor::Flavor,
        linker::Linker,
        ops::{Operator, OperatorErr},
    },
};

pub struct Mekh {}

#[derive(thiserror::Error, Debug)]
pub enum MekhError {}

impl<F: Flavor> Operator<F> for Mekh {
    type Error = MekhError;

    type Inverse = Kemh;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

pub struct Kemh {}

#[derive(thiserror::Error, Debug)]
pub enum KhmeError {}

impl<F: Flavor> Operator<F> for Kemh {
    type Error = KhmeError;

    type Inverse = Mekh;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

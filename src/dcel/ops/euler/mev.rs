use std::convert::Infallible;

use crate::dcel::{
    Dcel, Edge, EdgePtrs,
    flavor::Flavor,
    ops::{Operator, OperatorErr},
};

/// ad a vertex on an edge
pub struct Mev<F: Flavor> {
    pub edge: EdgePtrs,
    pub data: F::Edge,
}

impl<F: Flavor> Mev<F> {
    pub fn new(edge: EdgePtrs, data: F::Edge) -> Self {
        Self { edge, data }
    }
}

impl<F: Flavor> Operator<F> for Mev<F> {
    type Inverse = Kev;
    type Error = Infallible;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

pub struct Kev {}

impl<F: Flavor> Operator<F> for Kev {
    type Inverse = Mev<F>;
    type Error = Infallible;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

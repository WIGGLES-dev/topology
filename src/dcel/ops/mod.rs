use std::fmt::Debug;

use crate::{
    arena::Key,
    coord::FromCoordinate,
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, Face, FaceKey, FacePtrs, Keyed, Vertex, VertexKey,
        VertexPtrs, flavor::Flavor,
    },
};

mod combo;
mod euler;
mod geometry;

pub use combo::*;
pub use euler::*;
pub use geometry::*;

#[derive(thiserror::Error)]
pub struct OperatorErr<Op, Err> {
    pub op: Op,
    pub err: Err,
}

impl<Op, Err> Debug for OperatorErr<Op, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} failed", std::any::type_name::<Op>())
    }
}

pub trait Operator<F: Flavor>: Sized {
    type Check;
    type Error;
    type Inverse: Operator<F>;

    fn check(&self, dcel: &Dcel<F>) -> Result<Self::Check, Self::Error>;
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut Dcel<F>,
    ) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>>;
}

pub enum Error {}

pub enum Op<F: Flavor> {
    Euler(euler::EulerOp<F>),
    Geometry(geometry::GeometryOp),
}

impl<F: Flavor> Operator<F> for Op<F> {
    type Check = ();
    type Error = Error;
    type Inverse = Op<F>;
    fn check(&self, dcel: &Dcel<F>) -> Result<Self::Check, Self::Error> {
        todo!()
    }
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut Dcel<F>,
    ) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

pub struct CrdtDcel<F: Flavor> {
    version: usize,
    dcel: Dcel<F>,
    history: Vec<Op<F>>,
}

impl<F: Flavor> CrdtDcel<F> {
    pub fn new(dcel: Dcel<F>) -> Self {
        Self {
            version: 0,
            dcel,
            history: vec![],
        }
    }

    pub fn apply(&mut self, command: Command<F>) -> Result<(), OperatorErr<Op<F>, Error>> {
        match command.op.check(&self.dcel) {
            Ok(input) => {
                let inverse = command.op.apply(&input, &mut self.dcel)?;
                self.history.push(inverse);
                self.version += 1;
                Ok(())
            }
            Err(err) => Err(OperatorErr {
                op: command.op,
                err,
            }),
        }
    }
}

pub struct Command<F: Flavor> {
    version: usize,
    op: Op<F>,
}

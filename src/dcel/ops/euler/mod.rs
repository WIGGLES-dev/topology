//! Euler operators for a DCEL
//!
//! [mef]
//! Make/Kill an edge and a face.
//!
//! [mekh]
//!
//! [mev]
//!
//! [mevvf]
//!
//! [mve]
//!
//! [mveef]
//!
//! [mzev]
//!
//! [mzevmf]
//!

use crate::dcel::{
    Dcel, Flavor,
    ops::{Operator, OperatorErr},
};

/// make (kill) an edge, and a face
mod mef;
/// make (kill) an edge, and a hole (isolated vertex is considered hole)
mod mekh;
/// make (kill) a vertex and an edge (split)
mod mev;
/// make (kill) and edge and a vertex (append-stub)
mod mve;
/// make (kill) a vertex two edges and a face (zero area & zero perimeter)
mod mvh;
/// make (kill) an edge, two vertices, a face (zero area)
mod mvvef;

pub use mef::*;
pub use mekh::*;
pub use mev::*;
pub use mve::*;
pub use mvh::*;
pub use mvvef::*;

pub enum Error {}

pub enum EulerOp<F: Flavor> {
    Mef(Mef<F>),
    Mekh(Mekh),
    Kemh(Kemh),
    Kef(Kef),
    Mev(Mev<F>),
    Kev(Kev),
    Mvvef(Mvvef<F>),
    Kvvef(Kvvef),
    Mvh(Mvh<F>),
    Kvh(Kvh),
    Mve(Mve<F>),
    Kve(Kve),
}

macro_rules! into {
    ($op:ident) => {
        impl<F: Flavor> Into<EulerOp<F>> for $op {
            fn into(self) -> EulerOp<F> {
                EulerOp::$op(self)
            }
        }
    };
    ($op:ident<F>) => {
        impl<F: Flavor> Into<EulerOp<F>> for $op<F> {
            fn into(self) -> EulerOp<F> {
                EulerOp::$op(self)
            }
        }
    };
}

into!(Mef<F>);
into!(Mekh);
into!(Kemh);
into!(Kef);
into!(Mev<F>);
into!(Kev);
into!(Mvvef<F>);
into!(Kvvef);
into!(Mvh<F>);
into!(Kvh);
into!(Mve<F>);
into!(Kve);

impl<F: Flavor> super::Operator<F> for EulerOp<F> {
    type Check = ();
    type Error = Error;
    type Inverse = EulerOp<F>;
    fn check(&self, dcel: &crate::dcel::Dcel<F>) -> Result<Self::Check, Self::Error> {
        todo!()
    }
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut crate::dcel::Dcel<F>,
    ) -> Result<Self::Inverse, super::OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

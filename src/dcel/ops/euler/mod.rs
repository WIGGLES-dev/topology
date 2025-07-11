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

use std::fmt::Debug;

use crate::{
    arena::Key,
    coord::{Coordinate, FromCoordinate, UpdateCoordinate},
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

pub trait DataGenerator<F: Flavor> {
    fn make_vertex(self, dcel: &Dcel<F>) -> F::Vertex;
    fn make_edge(self, dcel: &Dcel<F>) -> F::Edge;
    fn make_face(self, dcel: &Dcel<F>) -> F::Face;
}

struct DefaultGenerator;
impl<F: Flavor> DataGenerator<F> for DefaultGenerator
where
    F::Vertex: Default,
    F::Edge: Default,
    F::Face: Default,
{
    fn make_vertex(self, dcel: &Dcel<F>) -> <F as Flavor>::Vertex {
        Default::default()
    }
    fn make_edge(self, dcel: &Dcel<F>) -> <F as Flavor>::Edge {
        Default::default()
    }
    fn make_face(self, dcel: &Dcel<F>) -> <F as Flavor>::Face {
        Default::default()
    }
}

impl<F: Flavor, GenV, GenE, GenF> DataGenerator<F> for (GenV, GenE, GenF)
where
    GenV: Fn() -> F::Vertex,
    GenE: Fn() -> F::Edge,
    GenF: Fn() -> F::Face,
{
    fn make_vertex(self, dcel: &Dcel<F>) -> <F as Flavor>::Vertex {
        (self.0)()
    }
    fn make_edge(self, dcel: &Dcel<F>) -> <F as Flavor>::Edge {
        (self.1)()
    }
    fn make_face(self, dcel: &Dcel<F>) -> <F as Flavor>::Face {
        (self.2)()
    }
}

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
    type Error;
    type Inverse: Operator<F>;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error>;
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>>;
}

macro_rules! op_group {
    (
        $vis:vis enum $name:ident<F: Flavor> {
            $( $variant:ident($op:ty) ),*
            $(,)?
        }

    ) => {
        $vis enum $name<F: Flavor> {
            $( $variant($op), )*
        }

        $(
            impl<F: Flavor> Into<$name<F>> for $op {
                fn into(self) -> $name<F> {
                    $name::$variant(self)
                }
            }
        )*

        $vis enum Error<F: Flavor>
        where F::Vertex: UpdateCoordinate + Coordinate
        {
            $(
                $variant(<$op as Operator<F>>::Error),
            )*
        }

        impl<F: Flavor> Operator<F> for $name<F>
        where F::Vertex: UpdateCoordinate + Coordinate
        {
            type Error = Error<F>;
            type Inverse = Op<F>;

            fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$variant(op) => op.check(dcel).map_err(Error::$variant),
                    )*
                }
            }
            fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
                match self {
                    $(
                        Self::$variant(op) =>
                            match op.apply(dcel) {
                                Ok(inverse) => Ok(inverse.into()),
                                Err(err) => Err(OperatorErr { op: err.op.into(), err: Error::$variant(err.err) })
                            },
                    )*
                }
            }
        }
    };
}

op_group!(
    pub enum Op<F: Flavor> {
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
        TranslateVertex(geometry::TranslateVertex),
        // CollapseEdge(combo::CollapseEdge),
        // UncollapseEdge(combo::UncollapseEdge<F>),
        // WeldVertex(combo::WeldVertex<F>),
        // UnweldVertex(combo::UnweldVertex<F>),
        // LinkVertices(combo::LinkVertices<F>),
        // UnlinkVertices(combo::UnlinkVertices),
    }
);

pub struct CrdtDcel<F: Flavor> {
    version: usize,
    dcel: Dcel<F>,
    history: Vec<Op<F>>,
}

impl<F: Flavor> CrdtDcel<F>
where
    F::Vertex: Coordinate,
{
    pub fn new(dcel: Dcel<F>) -> Self {
        Self {
            version: 0,
            dcel,
            history: vec![],
        }
    }

    pub fn apply(&mut self, command: Command<F>) -> Result<(), OperatorErr<Op<F>, Error<F>>>
    where
        F::Vertex: UpdateCoordinate + Coordinate,
    {
        match command.op.check(&self.dcel) {
            Ok(input) => {
                let inverse = command.op.apply(&mut self.dcel)?;
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

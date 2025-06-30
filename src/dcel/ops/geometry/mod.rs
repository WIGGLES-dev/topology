mod translate_face;
mod translate_vertex;

pub use translate_face::*;
pub use translate_vertex::*;

use crate::dcel::{Flavor, ops::Operator};

pub enum GeometryOp {
    TranslateVertex(TranslateVertex),
}

pub enum GeometryError {}

impl<F: Flavor> Operator<F> for GeometryOp {
    type Check = ();
    type Error = GeometryError;
    type Inverse = GeometryOp;
    fn check(&self, dcel: &crate::dcel::Dcel<F>) -> Result<Self::Check, Self::Error> {
        Ok(())
    }
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut crate::dcel::Dcel<F>,
    ) -> Result<Self::Inverse, super::OperatorErr<Self, Self::Error>> {
        todo!()
    }
}

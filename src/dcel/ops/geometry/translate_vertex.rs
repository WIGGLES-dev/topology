use crate::{
    arena::Key,
    coord::{Coordinate, UpdateCoordinate},
    dcel::{
        Dcel, Edge, EdgeKey, EdgePtrs, FaceKey, FacePtrs, Keyed, Traverser, Vertex, VertexKey,
        VertexPtrs,
        flavor::Flavor,
        linker::Linker,
        ops::{Operator, OperatorErr},
    },
};

pub struct TranslateVertex {
    vertex: Key<VertexKey>,
    coord: [f32; 3],
}

#[derive(thiserror::Error, Debug)]
pub enum TranslateVertexError {
    #[error("would make graph non planar")]
    WouldMakeNonPlanar,
}

impl<F: Flavor> Operator<F> for TranslateVertex
where
    F::Vertex: Coordinate + UpdateCoordinate,
{
    type Check = ();
    type Error = TranslateVertexError;
    type Inverse = TranslateVertex;

    fn check(&self, dcel: &Dcel<F>) -> Result<Self::Check, Self::Error> {
        Ok(())
    }
    fn apply(
        self,
        input: &Self::Check,
        dcel: &mut Dcel<F>,
    ) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let current_coord = self.vertex.weight(dcel).xyz();
        dcel.vertex_mut(self.vertex).weight.set_xyz(self.coord);
        Ok(TranslateVertex {
            vertex: self.vertex,
            coord: current_coord,
        })
    }
}

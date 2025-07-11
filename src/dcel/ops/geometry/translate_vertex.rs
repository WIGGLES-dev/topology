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
    pub vertex: Key<VertexKey>,
    pub delta: [f32; 3],
}

impl TranslateVertex {
    pub fn from_absolute<F: Flavor>(
        dcel: &Dcel<F>,
        key: Key<VertexKey>,
        absolute: impl Coordinate,
    ) -> Self
    where
        F::Vertex: Coordinate,
    {
        let [x, y, z] = key.weight(dcel).xyz();
        let [tx, ty, tz] = absolute.xyz();
        let delta = [tx - x, ty - y, tz - z];
        TranslateVertex { vertex: key, delta }
    }
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
    type Error = TranslateVertexError;
    type Inverse = TranslateVertex;

    fn check(&self, dcel: &Dcel<F>) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(self, dcel: &mut Dcel<F>) -> Result<Self::Inverse, OperatorErr<Self, Self::Error>> {
        let [x, y, z] = self.vertex.weight(dcel).xyz();
        let [dx, dy, dz] = self.delta.xyz();
        dcel.vertex_mut(self.vertex)
            .weight
            .set_xyz([x + dx, y + dy, z + dz]);
        Ok(TranslateVertex {
            vertex: self.vertex,
            delta: [-dx, -dy, -dz],
        })
    }
}

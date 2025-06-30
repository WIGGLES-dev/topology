use crate::{
    arena::Key,
    coord::{Coordinate, FromCoordinate},
    dcel::{Dcel, FaceKey, Flavor, VertexKey},
};

pub struct Draw<F: Flavor> {
    coord: [f32; 3],
    last_key: Key<VertexKey>,
    dcel: Dcel<F>,
}

impl<F: Flavor> Draw<F>
where
    F::Vertex: Coordinate + FromCoordinate,
    F::Edge: Default,
    F::Face: Default,
{
    pub fn new(
        mut dcel: Dcel<F>,
        v1: impl Coordinate,
        v2: impl Coordinate,
    ) -> (Self, [Key<VertexKey>; 2]) {
        let kvvef = dcel
            .mvvef(F::Vertex::from_xyz(v1.xyz()), F::Vertex::from_xyz(v2.xyz()))
            .unwrap();
        (
            Self {
                coord: v2.xyz(),
                last_key: kvvef.vertices[1],
                dcel,
            },
            kvvef.vertices,
        )
    }

    /// will panic if you have not called start
    pub fn key(&self) -> Key<VertexKey> {
        self.last_key
    }
    pub fn set_key(&mut self, key: Key<VertexKey>) {
        self.last_key = key
    }
    pub fn move_to(&mut self, coord: impl Coordinate) -> Key<VertexKey> {
        self.last_key = self
            .dcel
            .mvh(F::Vertex::from_xyz(coord.xyz()))
            .unwrap()
            .vertex;
        self.coord = coord.xyz();
        self.last_key
    }
    pub fn line_to(&mut self, coord: impl Coordinate) -> Key<VertexKey> {
        self.last_key = self
            .dcel
            .mve(self.last_key, F::Vertex::from_xyz(coord.xyz()))
            .unwrap()
            .vertex;
        self.coord = coord.xyz();
        self.last_key
    }
    pub fn close_path(&mut self, to: Key<VertexKey>) -> Key<FaceKey> {
        let face = self.dcel.mef(self.last_key, to).unwrap().face;
        self.last_key = to;
        face
    }

    pub fn finish(self) -> Dcel<F> {
        self.dcel
    }
}

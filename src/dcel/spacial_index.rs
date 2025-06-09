use crate::arena::Key;

use rstar::{
    RTree,
    primitives::{GeomWithData, Rectangle},
};

use super::{FaceKey, VertexKey};

/// spacial index for disconected vertices
#[derive(Default)]
pub struct VertexIndex {
    index: RTree<GeomWithData<[f32; 2], Key<VertexKey>>>,
}

/// spacial index for top level connected face components
#[derive(Default)]
pub struct FaceIndex {
    index: RTree<GeomWithData<Rectangle<[f32; 2]>, Key<FaceKey>>>,
}

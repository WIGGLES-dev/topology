use std::slice::GetDisjointMutError;

use super::{EdgeKey, FaceKey, Key, VertexKey};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    VertexDoesNotExist,
    EdgeDoesNotExist,
    FaceDoesNotExist,
    PlanarConflict,
    NoBoundingFace,
    NonDisjoint(GetDisjointMutError),
    DisconnectedVertex,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Option<Key<VertexKey>>> for Error {
    fn from(value: Option<Key<VertexKey>>) -> Self {
        Self::VertexDoesNotExist
    }
}

impl From<Option<Key<EdgeKey>>> for Error {
    fn from(value: Option<Key<EdgeKey>>) -> Self {
        Self::EdgeDoesNotExist
    }
}

impl From<Option<Key<FaceKey>>> for Error {
    fn from(value: Option<Key<FaceKey>>) -> Self {
        Self::FaceDoesNotExist
    }
}

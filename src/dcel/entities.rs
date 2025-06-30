use crate::{
    arena::{ArenaBitMask, Key},
    dcel::{Dcel, flavor::Flavor},
    weighted::Weighted,
};
use std::ops::{Deref, DerefMut};

pub struct VertexKey;
pub struct EdgeKey;
pub struct FaceKey;

pub trait HasKey {
    type Key;
}

pub struct Keyed<T>
where
    T: HasKey,
{
    pub(crate) inner: T,
    pub key: Key<T::Key>,
}

impl<T> Deref for Keyed<T>
where
    T: HasKey,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Keyed<T>
where
    T: HasKey,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct VertexPtrs {
    pub edge: Option<Key<EdgeKey>>,
}
impl HasKey for VertexPtrs {
    type Key = VertexKey;
}

pub type Vertex<W> = Weighted<VertexPtrs, W>;

pub struct EdgePtrs {
    pub origin: Key<VertexKey>,
    pub twin: Key<EdgeKey>,
    pub prev: Key<EdgeKey>,
    pub next: Key<EdgeKey>,
    pub face: Key<FaceKey>,
}
impl HasKey for EdgePtrs {
    type Key = EdgeKey;
}

pub type Edge<W> = Weighted<EdgePtrs, W>;

bitflags::bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct FaceMask: u8 {
        const IS_OUTER          = 0b0000_0001;
        const IS_BOUNDARY       = 0b0000_0010;
        const IS_ZERO_PERIMETER = 0b0000_0100;
        const IS_ZERO_AREA      = 0b0000_1000;

        const VISITED           = 0b0001_0000;
        const MARKED            = 0b0010_0000;
        const ACTIVE_REGION     = 0b0100_0000;
        const TEMP              = 0b1000_0000;
    }
}

pub enum HolRef {
    Face(Key<FaceKey>),
    Vertex(Key<VertexKey>),
}

pub struct FacePtrs {
    pub edge: Key<EdgeKey>,
    pub holes: Vec<HolRef>,
    pub mask: FaceMask,
}
impl HasKey for FacePtrs {
    type Key = FaceKey;
}

pub type Face<W> = Weighted<FacePtrs, W>;

impl<T> Face<T> {
    pub fn default_at_edge(edge: Key<EdgeKey>, mask: FaceMask) -> Self
    where
        T: Default,
    {
        Self {
            inner: FacePtrs {
                edge,
                holes: vec![],
                mask,
            },
            weight: T::default(),
        }
    }

    pub fn is_bounding(&self) -> bool {
        self.mask.contains(FaceMask::IS_BOUNDARY)
    }
}

macro_rules! convenience {
    ($key:ty => $method:ident.$prop:ident -> $rt:ty) => {
        impl Key<$key> {
            pub fn $prop<F: Flavor>(self, dcel: &Dcel<F>) -> $rt {
                dcel.$method(self).$prop
            }
        }
    };
    (ref $key:ty => $method:ident.$prop:ident -> $rt:ty) => {
        impl Key<$key> {
            pub fn $prop<F: Flavor>(self, dcel: &Dcel<F>) -> $rt {
                &dcel.$method(self).$prop
            }
        }
    };
    (ref mut $key:ty => $method:ident.$prop:ident -> $rt:ty) => {
        impl Key<$key> {
            pub fn $prop<V, E, F>(self, dcel: &Dcel<F>) -> $rt {
                &mut dcel.$method(self).$prop
            }
        }
    };
}

convenience!(VertexKey => vertex.edge -> Option<Key<EdgeKey>>);
convenience!(ref VertexKey => vertex.weight -> &F::Vertex);

convenience!(EdgeKey => edge.next -> Key<EdgeKey>);
convenience!(EdgeKey => edge.prev -> Key<EdgeKey>);
convenience!(EdgeKey => edge.twin -> Key<EdgeKey>);
convenience!(EdgeKey => edge.face -> Key<FaceKey>);
convenience!(EdgeKey => edge.origin -> Key<VertexKey>);
convenience!(ref EdgeKey => edge.weight -> &F::Edge);

convenience!(FaceKey => face.edge -> Key<EdgeKey>);
convenience!(ref FaceKey => face.weight -> &F::Face);
convenience!(FaceKey => face.mask -> FaceMask);

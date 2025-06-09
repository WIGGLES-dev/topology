use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU32,
    ops::Deref,
};

use serde::{Deserialize, Serialize};

pub struct Key<T = ()>(NonZeroU32, PhantomData<T>);

unsafe impl<T> Send for Key<T> {}
unsafe impl<T> Sync for Key<T> {}

impl<T> Deref for Key<T> {
    type Target = NonZeroU32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Serialize for Key<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.get().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Key<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(NonZeroU32::deserialize(deserializer)?, PhantomData))
    }
}

impl<T> std::fmt::Display for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl<T> PartialEq<Key<T>> for Key<T> {
    fn eq(&self, other: &Key<T>) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }
    fn ne(&self, other: &Key<T>) -> bool {
        PartialEq::ne(&self.0, &other.0)
    }
}

impl<T> Eq for Key<T> {}

impl<T> PartialOrd for Key<T> {
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(&self.0, &other.0)
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(&self.0, &other.0)
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(&self.0, &other.0)
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(&self.0, &other.0)
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.0, &other.0)
    }
}

impl<T> Ord for Key<T> {
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        Self(Ord::clamp(self.0, min.0, max.0), PhantomData)
    }
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.0, &other.0)
    }
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self(*Ord::max(&self.0, &other.0), PhantomData)
    }
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self(*Ord::min(&self.0, &other.0), PhantomData)
    }
}

impl<T> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.0, state);
    }
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for Key<T> {}

impl<T> Key<T> {
    pub fn new(id: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(id)?, PhantomData))
    }

    pub unsafe fn new_unchecked(id: u32) -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(id), PhantomData) }
    }

    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

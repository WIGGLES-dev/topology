use std::ops::{Deref, DerefMut};

pub struct Weighted<T, U> {
    weight: T,
    inner: U,
}

impl<T, U> AsRef<U> for Weighted<T, U> {
    fn as_ref(&self) -> &U {
        &self.inner
    }
}

impl<T, U> Deref for Weighted<T, U> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, U> DerefMut for Weighted<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

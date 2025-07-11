use std::ops::{Deref, DerefMut};

pub struct Weighted<T, W> {
    pub inner: T,
    pub weight: W,
}

impl<T, W> Deref for Weighted<T, W> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T, W> DerefMut for Weighted<T, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

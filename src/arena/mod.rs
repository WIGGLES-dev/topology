use std::{
    mem::ManuallyDrop,
    ops::{Index, IndexMut},
    slice::GetDisjointMutError,
};

mod key;
mod mask;

pub use key::Key;
pub use mask::ArenaBitMask;

pub enum Slot<T, K> {
    Filled(T),
    Empty(Key<K>),
}

pub struct Arena<T, K = ()> {
    inner: Vec<Option<T>>,
    free: Vec<u32>,
    next_key: Key<K>,
}

impl<T, K> Default for Arena<T, K> {
    fn default() -> Self {
        Self {
            inner: vec![None],
            free: vec![],
            next_key: unsafe { Key::new_unchecked(1) },
        }
    }
}

impl<T, K> Arena<T, K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_zero(zero: T) -> Self {
        Self {
            inner: vec![Some(zero)],
            ..Default::default()
        }
    }

    pub fn zero(&self) -> &Option<T> {
        &self.inner[0]
    }

    pub fn zero_mut(&mut self) -> &mut Option<T> {
        &mut self.inner[0]
    }

    pub fn from_iter(iter: impl Iterator<Item = T>) -> Self {
        let mut inner = Vec::new();
        inner.push(None);
        inner.extend(iter.map(Some));

        Self {
            inner,
            free: vec![],
            next_key: unsafe { Key::new_unchecked(1) },
        }
    }

    fn new_key(&mut self) -> Key<K> {
        let key = self.next_key;
        self.next_key = unsafe { Key::new_unchecked(key.get() + 1) };
        key
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn size(&self) -> usize {
        self.inner.len() - 1
    }

    pub fn get(&self, key: Key<K>) -> Option<&T> {
        match self.inner.get(key.get() as usize) {
            Some(Some(t)) => Some(t),
            _ => None,
        }
    }

    pub fn get_many<const N: usize>(&self, keys: [Key<K>; N]) -> [Option<&T>; N] {
        let mut out: [Option<&T>; N] = [None; N];
        for (i, key) in keys.iter().enumerate() {
            out[i] = self.get(*key);
        }
        return out;
    }

    pub fn get_mut(&mut self, key: Key<K>) -> Option<&mut T> {
        match self.inner.get_mut(key.get() as usize) {
            Some(Some(t)) => Some(t),
            _ => None,
        }
    }

    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        keys: [Key<K>; N],
    ) -> Result<[&mut Option<T>; N], GetDisjointMutError> {
        let mut indices: [usize; N] = [0; N];
        for (i, key) in keys.iter().enumerate() {
            indices[i] = key.get() as usize;
        }
        let v = self.inner.get_disjoint_mut(indices);

        v
    }

    pub unsafe fn get_disjoint_unchecked_mut<const N: usize>(
        &mut self,
        keys: [Key<K>; N],
    ) -> [&mut Option<T>; N] {
        let mut indices: [usize; N] = [0; N];
        for (i, key) in keys.iter().enumerate() {
            indices[i] = key.get() as usize;
        }
        let v = unsafe { self.inner.get_disjoint_unchecked_mut(indices) };

        v
    }

    pub fn set(&mut self, key: Key<K>, value: T) {
        let idx = key.get() as usize;
        if idx > self.len() - 1 {
            self.inner.resize_with(idx + 1, Default::default);
        }
        self.inner[idx] = Some(value);
    }

    pub fn remove(&mut self, key: Key<K>) -> Option<T> {
        let idx = key.get();
        if let Some(value) = self.inner.get_mut(key.get() as usize) {
            if let Some(value) = value.take() {
                self.free.push(idx);
                return Some(value);
            } else {
                return None;
            }
        }
        None
    }

    pub fn insert(&mut self, value: T) -> Key<K> {
        if let Some(idx) = self.free.pop() {
            self.inner[idx as usize] = Some(value);
            let key = unsafe { Key::new_unchecked(idx) };
            key
        } else {
            let key = self.new_key();
            self.inner.push(Some(value));
            key
        }
    }

    pub fn reserve(&mut self) -> Key<K> {
        if let Some(idx) = self.free.pop() {
            self.inner[idx as usize] = None;
            let key = unsafe { Key::new_unchecked(idx) };
            key
        } else {
            let key = self.new_key();
            self.inner.push(None);
            key
        }
    }

    pub fn key(&self, pos: u32) -> Option<Key<K>> {
        Key::new(pos)
    }

    pub fn ffi(&self, pos: u32) -> Option<&T> {
        match Key::new(pos) {
            Some(key) => self.get(key),
            None => None,
        }
    }

    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = (&'a T, Key<K>)> {
        self.inner
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(i, v)| match v {
                Some(v) => Some((v, self.key(i as u32)?)),
                None => None,
            })
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl 'a + Iterator<Item = (&'a mut T, Key<K>)> {
        self.inner
            .iter_mut()
            .enumerate()
            .skip(1)
            .filter_map(|(i, v)| match v {
                Some(v) => Some((v, unsafe { Key::new_unchecked(i as u32) })),
                None => None,
            })
    }
}

impl<T, K> Index<Key<K>> for Arena<T, K> {
    type Output = T;
    fn index(&self, index: Key<K>) -> &Self::Output {
        match self.get(index) {
            Some(v) => v,
            None => panic!("{} is out of bounds", index),
        }
    }
}

impl<T, K> IndexMut<Key<K>> for Arena<T, K> {
    fn index_mut(&mut self, index: Key<K>) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(v) => v,
            None => panic!(""),
        }
    }
}

use std::marker::PhantomData;

use crate::arena::{Arena, Key};

/// A compact visit flag for an arena more effecient than a HashSet
pub struct ArenaBitMask<K> {
    bits: Vec<u64>,
    phantom: PhantomData<K>,
}

impl<K> ArenaBitMask<K> {
    pub fn new<T>(arena: &Arena<T, K>) -> Self {
        let bits = Vec::with_capacity(arena.size() / 64);
        Self {
            bits,
            phantom: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.bits.fill(0);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bits: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    pub fn flip(&mut self, key: Key<K>) {
        let idx = key.get() - 1;
        let word = &mut self.bits[(idx / 64) as usize];
        let bit = idx % 64;
        *word ^= 1 << bit;
    }

    pub fn is_flipped(&self, key: Key<K>) -> bool {
        let idx = key.get() - 1;
        let word = self.bits[(idx / 64) as usize];
        let bit = idx % 64;
        word & 1 << bit != 0
    }
}

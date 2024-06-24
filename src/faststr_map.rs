use std::{any::TypeId, collections::hash_map::Entry};

use faststr::FastStr;
use rustc_hash::FxHashMapRand;

/// This is an optimized version of TypeMap to FastStr that eliminates the need to Box the values.
///
/// This map is suitable for T that impls both From<FastStr> and Into<FastStr>.
#[derive(Debug, Default)]
pub struct FastStrMap {
    inner: FxHashMapRand<TypeId, FastStr>,
}

impl FastStrMap {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: FxHashMapRand::default(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: FxHashMapRand::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, t: FastStr) {
        self.inner.insert(TypeId::of::<T>(), t);
    }

    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&FastStr> {
        self.inner.get(&TypeId::of::<T>())
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut FastStr> {
        self.inner.get_mut(&TypeId::of::<T>())
    }

    #[inline]
    pub fn contains<T: 'static>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn remove<T: 'static>(&mut self) -> Option<FastStr> {
        self.inner.remove(&TypeId::of::<T>())
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn extend(&mut self, other: FastStrMap) {
        self.inner.extend(other.inner)
    }

    #[inline]
    pub fn iter(&self) -> ::std::collections::hash_map::Iter<'_, TypeId, FastStr> {
        self.inner.iter()
    }

    #[inline]
    pub fn entry<T: 'static>(&mut self) -> Entry<'_, TypeId, FastStr> {
        self.inner.entry(TypeId::of::<T>())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

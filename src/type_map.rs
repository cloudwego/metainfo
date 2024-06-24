use std::{
    any::{Any, TypeId},
    collections::hash_map::Entry as MapEntry,
    marker::PhantomData,
};

use rustc_hash::FxHashMapRand;

pub(crate) type AnyObject = Box<dyn Any + Send + Sync>;

pub struct Entry<'a, K: 'a, V: 'a> {
    inner: MapEntry<'a, K, AnyObject>,
    _marker: PhantomData<V>,
}

impl<'a, K, V> Entry<'a, K, V> {
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V
    where
        V: Send + Sync + 'static,
    {
        let v = self.inner.or_insert_with(|| Box::new(default));
        v.downcast_mut().unwrap()
    }

    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V
    where
        V: Send + Sync + 'static,
    {
        let v = self.inner.or_insert_with(|| Box::new(default()));
        v.downcast_mut().unwrap()
    }

    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V
    where
        V: Send + Sync + 'static,
    {
        let v = self.inner.or_insert_with_key(|key| Box::new(default(key)));
        v.downcast_mut().unwrap()
    }

    #[inline]
    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Self
    where
        V: Send + Sync + 'static,
    {
        Entry {
            inner: self.inner.and_modify(|v| {
                f(v.downcast_mut().unwrap());
            }),
            _marker: PhantomData,
        }
    }

    #[allow(clippy::unwrap_or_default)]
    #[inline]
    pub fn or_default(self) -> &'a mut V
    where
        V: Default + Send + Sync + 'static,
    {
        self.or_insert_with(V::default)
    }
}

#[derive(Debug, Default)]
pub struct TypeMap {
    inner: FxHashMapRand<TypeId, AnyObject>,
}

impl TypeMap {
    #[inline]
    pub fn new() -> Self {
        TypeMap {
            inner: FxHashMapRand::default(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        TypeMap {
            inner: FxHashMapRand::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    #[inline]
    pub fn insert<T: Send + Sync + 'static>(&mut self, t: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(t));
    }

    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    #[inline]
    pub fn contains<T: 'static>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.inner
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok().map(|boxed| *boxed))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn extend(&mut self, other: TypeMap) {
        self.inner.extend(other.inner)
    }

    #[inline]
    pub fn iter(&self) -> ::std::collections::hash_map::Iter<'_, TypeId, AnyObject> {
        self.inner.iter()
    }

    #[inline]
    pub fn entry<T: 'static>(&mut self) -> Entry<'_, TypeId, T> {
        Entry {
            inner: self.inner.entry(TypeId::of::<T>()),
            _marker: PhantomData,
        }
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

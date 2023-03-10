use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use crate::Intermediate;

/// Type alias.
#[cfg(feature = "preserve-order")]
pub type MapImpl<K, V> = indexmap::IndexMap<K, V>;

/// Type alias.
#[cfg(not(feature = "preserve-order"))]
pub type MapImpl<K, V> = std::collections::HashMap<K, V>;

/// Map from string keys to `Intermediate` values.
///
/// It wraps the underlying map implementation and prohibits inlining of some
/// methods in order to make the generated code smaller.
#[derive(Debug, Clone)]
pub struct Map {
    inner: MapImpl<Cow<'static, str>, Intermediate>,
}

impl Map {
    /// Create a new map.
    #[inline(never)]
    pub fn new() -> Self {
        Self {
            inner: MapImpl::new(),
        }
    }

    /// Create a new map with a given capacity.
    #[inline(never)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: MapImpl::with_capacity(capacity),
        }
    }

    /// Get value associated with a given key.
    #[inline(never)]
    pub fn get(&self, key: &str) -> Option<&Intermediate> {
        self.inner.get(key)
    }

    /// Insert a given key-value pair into the map.
    #[inline(never)]
    pub fn insert_with_static_key(&mut self, key: &'static str, value: Intermediate) {
        self.inner.insert(Cow::Borrowed(key), value);
    }

    /// Insert a given key-value pair into the map.
    #[inline(never)]
    pub fn insert_with_owned_key(&mut self, key: String, value: Intermediate) {
        self.inner.insert(Cow::Owned(key), value);
    }
}

impl Default for Map {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<MapImpl<Cow<'static, str>, Intermediate>> for Map {
    #[inline]
    fn from(map: MapImpl<Cow<'static, str>, Intermediate>) -> Self {
        Self { inner: map }
    }
}

impl From<Map> for MapImpl<Cow<'static, str>, Intermediate> {
    #[inline]
    fn from(map: Map) -> Self {
        map.inner
    }
}

impl Deref for Map {
    type Target = MapImpl<Cow<'static, str>, Intermediate>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Map {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for Map {
    type Item = (Cow<'static, str>, Intermediate);

    #[cfg(feature = "preserve-order")]
    type IntoIter = indexmap::map::IntoIter<Cow<'static, str>, Intermediate>;

    #[cfg(not(feature = "preserve-order"))]
    type IntoIter = std::collections::hash_map::IntoIter<Cow<'static, str>, Intermediate>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'a Cow<'static, str>, &'a Intermediate);

    #[cfg(feature = "preserve-order")]
    type IntoIter = indexmap::map::Iter<'a, Cow<'static, str>, Intermediate>;

    #[cfg(not(feature = "preserve-order"))]
    type IntoIter = std::collections::hash_map::Iter<'a, Cow<'static, str>, Intermediate>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

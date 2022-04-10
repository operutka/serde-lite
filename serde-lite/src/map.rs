use std::ops::{Deref, DerefMut};

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
/// methods in order to make the generated code much smaller.
#[derive(Debug, Clone)]
pub struct Map {
    inner: MapImpl<String, Intermediate>,
}

impl Map {
    /// Create a new map.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: MapImpl::new(),
        }
    }

    /// Create a new map with a given capacity.
    #[inline]
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
    ///
    /// The key is converted into its owned representation within this method
    /// and the method also drops any previous value. In combination with
    /// disabled inlining, it allows to create much smaller serializer methods.
    #[inline(never)]
    pub fn insert_with_str_key(&mut self, key: &str, value: Intermediate) {
        self.inner.insert(String::from(key), value);
    }
}

impl Default for Map {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<MapImpl<String, Intermediate>> for Map {
    #[inline]
    fn from(map: MapImpl<String, Intermediate>) -> Self {
        Self { inner: map }
    }
}

impl From<Map> for MapImpl<String, Intermediate> {
    #[inline]
    fn from(map: Map) -> Self {
        map.inner
    }
}

impl Deref for Map {
    type Target = MapImpl<String, Intermediate>;

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
    type Item = (String, Intermediate);

    #[cfg(feature = "preserve-order")]
    type IntoIter = indexmap::map::IntoIter<String, Intermediate>;

    #[cfg(not(feature = "preserve-order"))]
    type IntoIter = std::collections::hash_map::IntoIter<String, Intermediate>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'a String, &'a Intermediate);

    #[cfg(feature = "preserve-order")]
    type IntoIter = indexmap::map::Iter<'a, String, Intermediate>;

    #[cfg(not(feature = "preserve-order"))]
    type IntoIter = std::collections::hash_map::Iter<'a, String, Intermediate>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

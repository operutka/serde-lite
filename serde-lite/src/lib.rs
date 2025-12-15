//! This library provides a bit more lightweight implementation (compared to Serde)
//! of general-purpose serialization and de-serialization. **The intention here is
//! not to replace Serde completely** though. Serde does an amazing job and it
//! wouldn't make much sense to compete with Serde in terms of
//! serialization/de-serialization speed and the amount of memory used in runtime.
//!
//! We focus mainly on the one thing where Serde can be a pain in the... code :) -
//! and it's the size of the resulting binary. Depending on the complexity of the
//! types that you try to serialize/de-serialize, Serde can produce quite a lot of
//! code. Plus there is also some monomorphization which may add even more code to
//! your binary.
//!
//! In order to achieve it's goal, this library does some assumptions about the
//! underlying format. It uses an intermediate data representation that is similar
//! to JSON. The intermediate format can be then serialized/deserialized using
//! Serde. This implies that this library will never be as fast as Serde itself.
//!
//! # Usage
//!
//! You can use this library as a drop-in replacement for Serde. There are
//! `Serialize` and `Deserialize` traits that can be automatically derived and
//! there are also some attributes (compatible with Serde), so all you really have
//! to do is to put `serde-lite` instead of `serde` into your `Cargo.toml`.
//!
//! ## Serialization
//!
//! Here is a brief example of serialization into JSON:
//! ```rust
//! use serde_lite::Serialize;
//! use serde_lite_derive::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyStruct {
//!     field1: u32,
//!     field2: String,
//! }
//!
//! let instance = MyStruct {
//!     field1: 10,
//!     field2: String::from("Hello, World!"),
//! };
//!
//! let intermediate = instance.serialize().unwrap();
//! let json = serde_json::to_string_pretty(&intermediate).unwrap();
//! ```
//!
//! ## De-serialization
//!
//! Here is a brief example of de-serialization from JSON:
//! ```rust
//! use serde_lite::Deserialize;
//! use serde_lite_derive::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct MyStruct {
//!     field1: u32,
//!     field2: String,
//! }
//!
//! let input = r#"{
//!     "field1": 10,
//!     "field2": "Hello, World!"
//! }"#;
//!
//! let intermediate = serde_json::from_str(input).unwrap();
//! let instance = MyStruct::deserialize(&intermediate).unwrap();
//! ```
//!
//! ## Update
//!
//! Wait. What? Yes, this library has one more cool feature - partial updates.
//! Simply derive `Update` the same way you'd derive `Deserialize`. Example:
//! ```rust
//! use serde_lite::{Deserialize, Update};
//! use serde_lite_derive::{Deserialize, Update};
//!
//! #[derive(Deserialize, Update)]
//! struct MyStruct {
//!     field1: u32,
//!     field2: String,
//! }
//!
//! let mut instance = MyStruct {
//!     field1: 10,
//!     field2: String::new(),
//! };
//!
//! let input = r#"{
//!     "field2": "Hello, World!"
//! }"#;
//!
//! let intermediate = serde_json::from_str(input).unwrap();
//! let instance = instance.update(&intermediate).unwrap();
//! ```
//!
//! This feature can be especially handy if you're constructing a REST API and
//! you'd like to allow partial updates of your data.
//!
//! ## Supported attributes
//!
//! The library does not support all Serde attributes at this moment. Patches are
//! definitely welcome. These attributes are supported:
//!
//! * Container attributes:
//!     * `tag`
//!     * `content`
//! * Field attributes:
//!     * `default`
//!     * `flatten`
//!     * `rename`
//!     * `skip`
//!     * `skip_serializing`
//!     * `skip_serializing_if`
//!     * `skip_deserializing`
//!     * `serialize_with`
//!     * `deserialize_with`
//!     * `update_with`
//! * Enum variant attributes:
//!     * `rename`
//!
//! # When to use this library
//!
//! You can use this library whenever you need to serialize/de-serialize some
//! complex types and the size of the resulting binary matters to you. It is also
//! very useful in projects where you need to be able to partially update your data
//! based on the user input (e.g. REST APIs).
//!
//! # When to avoid using this library
//!
//! If the only thing that matters to you is the runtime performance, you probably
//! don't want to use this library. It also isn't very useful for
//! serializing/de-serializing huge amount of data because it needs to be
//! transformed into the intermediate representation at first. And, finally, this
//! library can only be used with self-describing formats like JSON.

mod deserialize;
mod intermediate;
mod map;
mod serialize;
mod update;

use std::{
    borrow::Cow,
    collections::LinkedList,
    fmt::{self, Display, Formatter},
};

#[cfg(feature = "derive")]
pub use serde_lite_derive::{Deserialize, Serialize, Update};

pub use crate::{
    deserialize::Deserialize,
    intermediate::{Intermediate, Number},
    map::{Map, MapImpl},
    serialize::Serialize,
    update::Update,
};

/// Error.
#[derive(Debug, Clone)]
pub enum Error {
    OutOfBounds,
    UnsupportedConversion,
    MissingField,
    UnknownEnumVariant,
    MissingEnumVariantContent,
    InvalidValue(Cow<'static, str>),
    NamedFieldErrors(ErrorList<NamedFieldError>),
    UnnamedFieldErrors(ErrorList<UnnamedFieldError>),
    Custom(Cow<'static, str>),
}

impl Error {
    /// Create an invalid value error with a given expected type name.
    #[inline]
    pub fn invalid_value<T>(expected: T) -> Self
    where
        T: ToString,
    {
        Self::InvalidValue(Cow::Owned(expected.to_string()))
    }

    /// Create an invalid value error with a given expected type name.
    #[inline]
    pub const fn invalid_value_static(expected: &'static str) -> Self {
        Self::InvalidValue(Cow::Borrowed(expected))
    }

    /// Create a custom error with a given error message.
    #[inline]
    pub fn custom<T>(msg: T) -> Self
    where
        T: ToString,
    {
        Self::Custom(Cow::Owned(msg.to_string()))
    }

    /// Create a custom error with a given error message.
    #[inline]
    pub const fn custom_static(msg: &'static str) -> Self {
        Self::Custom(Cow::Borrowed(msg))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::OutOfBounds => f.write_str("value is out of bounds"),
            Self::UnsupportedConversion => f.write_str("conversion not supported"),
            Self::MissingField => f.write_str("missing field"),
            Self::UnknownEnumVariant => f.write_str("unknown enum variant"),
            Self::MissingEnumVariantContent => f.write_str("missing enum variant content"),
            Self::InvalidValue(expected) => write!(f, "invalid value ({} expected)", expected),
            Self::NamedFieldErrors(errors) => {
                write!(f, "field errors ({})", errors)
            }
            Self::UnnamedFieldErrors(errors) => {
                write!(f, "field errors ({})", errors)
            }
            Self::Custom(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorList<NamedFieldError>> for Error {
    #[inline]
    fn from(errors: ErrorList<NamedFieldError>) -> Self {
        Self::NamedFieldErrors(errors)
    }
}

impl From<NamedFieldError> for Error {
    #[inline]
    fn from(err: NamedFieldError) -> Self {
        let mut errors = ErrorList::new();

        errors.push(err);

        Self::from(errors)
    }
}

impl From<ErrorList<UnnamedFieldError>> for Error {
    #[inline]
    fn from(errors: ErrorList<UnnamedFieldError>) -> Self {
        Self::UnnamedFieldErrors(errors)
    }
}

impl From<UnnamedFieldError> for Error {
    #[inline]
    fn from(err: UnnamedFieldError) -> Self {
        let mut errors = ErrorList::new();

        errors.push(err);

        Self::from(errors)
    }
}

/// Error associated with a named field.
#[derive(Debug, Clone)]
pub struct NamedFieldError {
    field: Cow<'static, str>,
    error: Error,
}

impl NamedFieldError {
    /// Create a new error for a given named field.
    #[inline]
    pub fn new<T>(field: T, error: Error) -> Self
    where
        T: ToString,
    {
        Self {
            field: Cow::Owned(field.to_string()),
            error,
        }
    }

    /// Create a new error for a given named field.
    #[inline]
    pub const fn new_static(field: &'static str, error: Error) -> Self {
        Self {
            field: Cow::Borrowed(field),
            error,
        }
    }

    /// Get the name of the field.
    #[inline]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Get the error.
    #[inline]
    pub fn error(&self) -> &Error {
        &self.error
    }

    /// Take the error.
    #[inline]
    pub fn into_error(self) -> Error {
        self.error
    }
}

impl Display for NamedFieldError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

impl std::error::Error for NamedFieldError {}

/// Error associated with an unnamed field.
#[derive(Debug, Clone)]
pub struct UnnamedFieldError {
    index: usize,
    error: Error,
}

impl UnnamedFieldError {
    /// Create a new error for a given field.
    #[inline]
    pub const fn new(field_index: usize, error: Error) -> Self {
        Self {
            index: field_index,
            error,
        }
    }

    /// Get index of the field.
    #[inline]
    pub fn field_index(&self) -> usize {
        self.index
    }

    /// Get the error.
    #[inline]
    pub fn error(&self) -> &Error {
        &self.error
    }

    /// Take the error.
    #[inline]
    pub fn into_error(self) -> Error {
        self.error
    }
}

impl Display for UnnamedFieldError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.index, self.error)
    }
}

impl std::error::Error for UnnamedFieldError {}

/// List of errors.
#[derive(Debug, Clone)]
pub struct ErrorList<T> {
    inner: LinkedList<T>,
}

impl<T> ErrorList<T> {
    /// Create a new error list.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: LinkedList::new(),
        }
    }

    /// Check if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get length of the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Add a given error to the list.
    #[inline]
    pub fn push(&mut self, err: T) {
        self.inner.push_back(err)
    }

    /// Append a given list of errors to the current one.
    #[inline]
    pub fn append(&mut self, mut other: Self) {
        self.inner.append(&mut other.inner)
    }

    /// Iterate over the errors.
    #[inline]
    pub fn iter(&self) -> std::collections::linked_list::Iter<'_, T> {
        self.inner.iter()
    }
}

impl<T> Default for ErrorList<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a ErrorList<T> {
    type Item = &'a T;
    type IntoIter = std::collections::linked_list::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<T> IntoIterator for ErrorList<T> {
    type Item = T;
    type IntoIter = std::collections::linked_list::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T> Display for ErrorList<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut iter = self.iter();

        if let Some(first) = iter.next() {
            Display::fmt(first, f)?;
        }

        for err in iter {
            write!(f, ", {}", err)?;
        }

        Ok(())
    }
}

impl<T> std::error::Error for ErrorList<T> where T: std::error::Error {}

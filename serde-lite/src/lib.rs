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
mod serialize;
mod update;

use std::fmt::{self, Display, Formatter};

#[cfg(feature = "derive")]
pub use serde_lite_derive::{Deserialize, Serialize, Update};

pub use crate::{
    deserialize::Deserialize,
    intermediate::{Intermediate, Map, Number},
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
    InvalidKey(String),
    InvalidValue(String),
    NamedFieldErrors(Vec<(String, Error)>),
    UnnamedFieldErrors(Vec<(usize, Error)>),
    Custom(String),
}

impl Error {
    /// Create an invalid value error with a given expected type name.
    pub fn invalid_value<T>(expected: T) -> Self
    where
        T: ToString,
    {
        Self::InvalidValue(expected.to_string())
    }

    /// Create a custom error with a given error message.
    pub fn custom<T>(msg: T) -> Self
    where
        T: ToString,
    {
        Self::Custom(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::OutOfBounds => f.write_str("value is out of bounds"),
            Self::UnsupportedConversion => f.write_str("conversion not supported"),
            Self::MissingField => f.write_str("missing field"),
            Self::UnknownEnumVariant => f.write_str("unknown enum variant"),
            Self::MissingEnumVariantContent => f.write_str("missing enum variant content"),
            Self::InvalidKey(msg) => write!(f, "invalid key: {}", msg),
            Self::InvalidValue(expected) => write!(f, "invalid value ({} expected)", expected),
            Self::NamedFieldErrors(errors) => {
                write!(f, "field errors (")?;
                for (index, (name, error)) in errors.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", {}: {}", name, error)?;
                    } else {
                        write!(f, "{}: {}", name, error)?;
                    }
                }
                write!(f, ")")
            }
            Self::UnnamedFieldErrors(errors) => {
                write!(f, "field errors (")?;
                for (error_index, (field_index, error)) in errors.iter().enumerate() {
                    if error_index > 0 {
                        write!(f, ", {}: {}", field_index, error)?;
                    } else {
                        write!(f, "{}: {}", field_index, error)?;
                    }
                }
                write!(f, ")")
            }
            Self::Custom(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

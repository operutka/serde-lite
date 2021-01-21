//! # Serde Lite

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

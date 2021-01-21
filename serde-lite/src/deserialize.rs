use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    convert::TryInto,
    fmt::Display,
    hash::Hash,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{Error, Intermediate};

/// Deserialize trait.
///
/// The trait can be implemented by objects the can deserialized from the
/// intermediate representation.
pub trait Deserialize {
    /// Deserialize an object instance.
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Deserialize for bool {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_bool().ok_or_else(|| Error::invalid_value("bool"))
    }
}

macro_rules! deserialize_for_signed_int {
    ( $x:ty ) => {
        impl Deserialize for $x {
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                val.as_i64()
                    .ok_or_else(|| Error::invalid_value("integer"))??
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)
            }
        }
    };
}

macro_rules! deserialize_for_unsigned_int {
    ( $x:ty ) => {
        impl Deserialize for $x {
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                val.as_u64()
                    .ok_or_else(|| Error::invalid_value("unsigned integer"))??
                    .try_into()
                    .map_err(|_| Error::OutOfBounds)
            }
        }
    };
}

deserialize_for_signed_int!(i8);
deserialize_for_signed_int!(i16);
deserialize_for_signed_int!(i32);
deserialize_for_signed_int!(isize);

deserialize_for_unsigned_int!(u8);
deserialize_for_unsigned_int!(u16);
deserialize_for_unsigned_int!(u32);
deserialize_for_unsigned_int!(usize);

impl Deserialize for i64 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_i64()
            .ok_or_else(|| Error::invalid_value("integer"))?
    }
}

impl Deserialize for u64 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_u64()
            .ok_or_else(|| Error::invalid_value("unsigned integer"))?
    }
}

impl Deserialize for i128 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_i64()
            .ok_or_else(|| Error::invalid_value("integer"))?
            .map(|v| v.into())
    }
}

impl Deserialize for u128 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_u64()
            .ok_or_else(|| Error::invalid_value("unsigned integer"))?
            .map(|v| v.into())
    }
}

impl Deserialize for f32 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_f64()
            .map(|v| v as _)
            .ok_or_else(|| Error::invalid_value("number"))
    }
}

impl Deserialize for f64 {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_f64().ok_or_else(|| Error::invalid_value("number"))
    }
}

impl Deserialize for char {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_char()
            .ok_or_else(|| Error::invalid_value("character"))
    }
}

impl Deserialize for String {
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_str()
            .map(|v| v.to_string())
            .ok_or_else(|| Error::invalid_value("string"))
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if val.is_none() {
            Ok(None)
        } else {
            T::deserialize(val).map(Some)
        }
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(val) = val.as_array() {
            let mut res = Vec::with_capacity(val.len());

            for elem in val {
                res.push(T::deserialize(elem)?);
            }

            Ok(res)
        } else {
            Err(Error::invalid_value("array"))
        }
    }
}

impl<T> Deserialize for [T; 0] {
    fn deserialize(_: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok([])
    }
}

macro_rules! deserialize_array {
    ( $len:expr => ($($n:tt)+) ) => {
        impl<T> Deserialize for [T; $len]
        where
            T: Deserialize,
        {
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                if let Some(val) = val.as_array() {
                    if val.len() < $len {
                        return Err(Error::invalid_value(concat!("an array of length ", $len)));
                    }

                    Ok([
                        $(
                            T::deserialize(&val[$n])?
                        ),+
                    ])
                } else {
                    Err(Error::invalid_value(concat!("an array of length ", $len)))
                }
            }
        }
    };
}

deserialize_array!(1 => (0));
deserialize_array!(2 => (0 1));
deserialize_array!(3 => (0 1 2));
deserialize_array!(4 => (0 1 2 3));
deserialize_array!(5 => (0 1 2 3 4));
deserialize_array!(6 => (0 1 2 3 4 5));
deserialize_array!(7 => (0 1 2 3 4 5 6));
deserialize_array!(8 => (0 1 2 3 4 5 6 7));
deserialize_array!(9 => (0 1 2 3 4 5 6 7 8));
deserialize_array!(10 => (0 1 2 3 4 5 6 7 8 9));
deserialize_array!(11 => (0 1 2 3 4 5 6 7 8 9 10));
deserialize_array!(12 => (0 1 2 3 4 5 6 7 8 9 10 11));
deserialize_array!(13 => (0 1 2 3 4 5 6 7 8 9 10 11 12));
deserialize_array!(14 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13));
deserialize_array!(15 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14));
deserialize_array!(16 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15));
deserialize_array!(17 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16));
deserialize_array!(18 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17));
deserialize_array!(19 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18));
deserialize_array!(20 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19));
deserialize_array!(21 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20));
deserialize_array!(22 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21));
deserialize_array!(23 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22));
deserialize_array!(24 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23));
deserialize_array!(25 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24));
deserialize_array!(26 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25));
deserialize_array!(27 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26));
deserialize_array!(28 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27));
deserialize_array!(29 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28));
deserialize_array!(30 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29));
deserialize_array!(31 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30));
deserialize_array!(32 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31));

impl Deserialize for () {
    fn deserialize(_: &Intermediate) -> Result<Self, Error> {
        Ok(())
    }
}

macro_rules! deserialize_tuple {
    ( $len:expr => ($($n:tt $ty:ident)+) ) => {
        impl<$($ty),+> Deserialize for ($($ty,)+)
        where
            $($ty: Deserialize,)+
        {
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                if let Some(val) = val.as_array() {
                    if val.len() < $len {
                        return Err(Error::invalid_value(concat!("an array of length ", $len)));
                    }

                    Ok((
                        $(
                            $ty::deserialize(&val[$n])?,
                        )+
                    ))
                } else {
                    Err(Error::invalid_value(concat!("an array of length ", $len)))
                }
            }
        }
    };
}

deserialize_tuple!(1 => (0 T0));
deserialize_tuple!(2 => (0 T0 1 T1));
deserialize_tuple!(3 => (0 T0 1 T1 2 T2));
deserialize_tuple!(4 => (0 T0 1 T1 2 T2 3 T3));
deserialize_tuple!(5 => (0 T0 1 T1 2 T2 3 T3 4 T4));
deserialize_tuple!(6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5));
deserialize_tuple!(7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6));
deserialize_tuple!(8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7));
deserialize_tuple!(9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8));
deserialize_tuple!(10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9));
deserialize_tuple!(11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10));
deserialize_tuple!(12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11));
deserialize_tuple!(13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12));
deserialize_tuple!(14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13));
deserialize_tuple!(15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14));
deserialize_tuple!(16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15));

impl<K, V> Deserialize for HashMap<K, V>
where
    K: FromStr + Eq + Hash,
    K::Err: Display,
    V: Deserialize,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let val = val.as_map().ok_or_else(|| Error::invalid_value("map"))?;

        let mut res = HashMap::with_capacity(val.len());

        for (name, value) in val {
            let k = K::from_str(&name).map_err(|err| Error::InvalidKey(err.to_string()))?;
            let v = V::deserialize(&value)?;

            res.insert(k, v);
        }

        Ok(res)
    }
}

#[cfg(feature = "preserve-order")]
impl<K, V> Deserialize for indexmap::IndexMap<K, V>
where
    K: FromStr + Eq + Hash,
    K::Err: Display,
    V: Deserialize,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let val = val.as_map().ok_or_else(|| Error::invalid_value("map"))?;

        let mut res = indexmap::IndexMap::with_capacity(val.len());

        for (name, value) in val {
            let k = K::from_str(&name).map_err(|err| Error::InvalidKey(err.to_string()))?;
            let v = V::deserialize(&value)?;

            res.insert(k, v);
        }

        Ok(res)
    }
}

macro_rules! deserialize_wrapper {
    ( $x:ident ) => {
        impl<T> Deserialize for $x<T>
        where
            T: Deserialize,
        {
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                let inner = T::deserialize(val)?;

                Ok($x::new(inner))
            }
        }
    };
}

deserialize_wrapper!(Box);
deserialize_wrapper!(Rc);
deserialize_wrapper!(Arc);
deserialize_wrapper!(Cell);
deserialize_wrapper!(RefCell);
deserialize_wrapper!(Mutex);

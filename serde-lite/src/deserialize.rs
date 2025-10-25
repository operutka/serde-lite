use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::HashMap,
    convert::TryInto,
    hash::Hash,
    rc::Rc,
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
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_bool()
            .ok_or_else(|| Error::invalid_value_static("bool"))
    }
}

macro_rules! deserialize_for_signed_int {
    ( $x:ty ) => {
        impl Deserialize for $x {
            #[inline]
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                val.as_number()
                    .ok_or_else(|| Error::invalid_value_static("integer"))
                    .and_then(|n| n.try_into())
            }
        }
    };
}

macro_rules! deserialize_for_unsigned_int {
    ( $x:ty ) => {
        impl Deserialize for $x {
            #[inline]
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                val.as_number()
                    .ok_or_else(|| Error::invalid_value_static("unsigned integer"))
                    .and_then(|n| n.try_into())
            }
        }
    };
}

deserialize_for_signed_int!(i8);
deserialize_for_signed_int!(i16);
deserialize_for_signed_int!(i32);
deserialize_for_signed_int!(i64);
deserialize_for_signed_int!(isize);

deserialize_for_unsigned_int!(u8);
deserialize_for_unsigned_int!(u16);
deserialize_for_unsigned_int!(u32);
deserialize_for_unsigned_int!(u64);
deserialize_for_unsigned_int!(usize);

impl Deserialize for i128 {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        i64::deserialize(val).map(|v| v.into())
    }
}

impl Deserialize for u128 {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        u64::deserialize(val).map(|v| v.into())
    }
}

impl Deserialize for f32 {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        f64::deserialize(val).map(|v| v as _)
    }
}

impl Deserialize for f64 {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_number()
            .map(|n| n.into())
            .ok_or_else(|| Error::invalid_value_static("number"))
    }
}

impl Deserialize for char {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_char()
            .ok_or_else(|| Error::invalid_value_static("character"))
    }
}

impl Deserialize for String {
    #[inline]
    fn deserialize(val: &Intermediate) -> Result<Self, Error> {
        val.as_str()
            .map(String::from)
            .ok_or_else(|| Error::invalid_value_static("string"))
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    #[inline]
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
            Err(Error::invalid_value_static("array"))
        }
    }
}

impl<T> Deserialize for [T; 0] {
    #[inline]
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
                        return Err(Error::invalid_value_static(concat!("an array of length ", $len)));
                    }

                    Ok([
                        $(
                            T::deserialize(&val[$n])?
                        ),+
                    ])
                } else {
                    Err(Error::invalid_value_static(concat!("an array of length ", $len)))
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
    #[inline]
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
                        return Err(Error::invalid_value_static(concat!("an array of length ", $len)));
                    }

                    Ok((
                        $(
                            $ty::deserialize(&val[$n])?,
                        )+
                    ))
                } else {
                    Err(Error::invalid_value_static(concat!("an array of length ", $len)))
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

impl<K, V, S> Deserialize for HashMap<K, V, S>
where
    K: TryFrom<Cow<'static, str>> + Eq + Hash,
    <K as TryFrom<Cow<'static, str>>>::Error: std::fmt::Display,
    V: Deserialize,
    S: core::hash::BuildHasher + Default,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let val = val
            .as_map()
            .ok_or_else(|| Error::invalid_value_static("map"))?;

        let mut res = HashMap::with_capacity_and_hasher(val.len(), S::default());

        for (name, value) in val {
            let k = K::try_from(name.clone()).map_err(Error::invalid_value)?;
            let v = V::deserialize(value)?;

            res.insert(k, v);
        }

        Ok(res)
    }
}

#[cfg(feature = "preserve-order")]
impl<K, V, S> Deserialize for indexmap::IndexMap<K, V, S>
where
    K: TryFrom<Cow<'static, str>> + Eq + Hash,
    <K as TryFrom<Cow<'static, str>>>::Error: std::fmt::Display,
    V: Deserialize,
    S: core::hash::BuildHasher + Default,
{
    fn deserialize(val: &Intermediate) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let val = val
            .as_map()
            .ok_or_else(|| Error::invalid_value_static("map"))?;

        let mut res = indexmap::IndexMap::with_capacity_and_hasher(val.len(), S::default());

        for (name, value) in val {
            let k = K::try_from(name.clone()).map_err(Error::invalid_value)?;
            let v = V::deserialize(value)?;

            res.insert(k, v);
        }

        Ok(res)
    }
}

macro_rules! deserialize_wrapper {
    ( $x:ident ) => {
        impl<T> Deserialize for $x<T>
        where
            T: Deserialize + Sized,
        {
            #[inline]
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

macro_rules! deserialize_wrapped_array {
    ( $x:ident ) => {
        impl<T> Deserialize for $x<[T]>
        where
            T: Deserialize,
            Vec<T>: Into<$x<[T]>>,
        {
            #[inline]
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                let inner = Vec::<T>::deserialize(val)?;
                Ok(inner.into())
            }
        }
        impl Deserialize for $x<str> {
            #[inline]
            fn deserialize(val: &Intermediate) -> Result<Self, Error> {
                let inner = String::deserialize(val)?;
                Ok(inner.into())
            }
        }
    };
}

deserialize_wrapped_array!(Box);
deserialize_wrapped_array!(Rc);
deserialize_wrapped_array!(Arc);

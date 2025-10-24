use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    convert::TryFrom,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{Error, Intermediate, Map, Number};

/// Serialize trait.
///
/// The trait can be implemented by objects the can serialized to the
/// intermediate representation.
pub trait Serialize {
    /// Serialize the object.
    fn serialize(&self) -> Result<Intermediate, Error>;
}

impl Serialize for bool {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Bool(*self))
    }
}

impl Serialize for i64 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Number(Number::SignedInt(*self)))
    }
}

impl Serialize for u64 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Number(Number::UnsignedInt(*self)))
    }
}

impl Serialize for f32 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Number(Number::Float(*self as _)))
    }
}

impl Serialize for f64 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Number(Number::Float(*self)))
    }
}

macro_rules! serialize_for_signed_int {
    ( $x:ty ) => {
        impl Serialize for $x {
            #[inline]
            fn serialize(&self) -> Result<Intermediate, Error> {
                Ok(Intermediate::Number(Number::SignedInt(i64::from(*self))))
            }
        }
    };
}

macro_rules! serialize_for_unsigned_int {
    ( $x:ty ) => {
        impl Serialize for $x {
            #[inline]
            fn serialize(&self) -> Result<Intermediate, Error> {
                Ok(Intermediate::Number(Number::UnsignedInt(u64::from(*self))))
            }
        }
    };
}

serialize_for_signed_int!(i8);
serialize_for_signed_int!(i16);
serialize_for_signed_int!(i32);

serialize_for_unsigned_int!(u8);
serialize_for_unsigned_int!(u16);
serialize_for_unsigned_int!(u32);

impl Serialize for i128 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        i64::try_from(*self)
            .map(|v| Intermediate::Number(Number::SignedInt(v)))
            .map_err(|_| Error::OutOfBounds)
    }
}

impl Serialize for u128 {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        u64::try_from(*self)
            .map(|v| Intermediate::Number(Number::UnsignedInt(v)))
            .map_err(|_| Error::OutOfBounds)
    }
}

impl Serialize for isize {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        i64::try_from(*self)
            .map(|v| Intermediate::Number(Number::SignedInt(v)))
            .map_err(|_| Error::OutOfBounds)
    }
}

impl Serialize for usize {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        u64::try_from(*self)
            .map(|v| Intermediate::Number(Number::UnsignedInt(v)))
            .map_err(|_| Error::OutOfBounds)
    }
}

impl Serialize for char {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::String(Cow::Owned(self.to_string())))
    }
}

impl Serialize for String {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::String(Cow::Owned(self.clone())))
    }
}

impl<'a> Serialize for &'a str {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::String(Cow::Owned(String::from(*self))))
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        if let Some(inner) = self.as_ref() {
            inner.serialize()
        } else {
            Ok(Intermediate::None)
        }
    }
}

impl<'a, T> Serialize for &'a [T]
where
    T: Serialize,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        serialize_slice(self)
    }
}

impl<'a, T> Serialize for &'a mut [T]
where
    T: Serialize,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        serialize_slice(self)
    }
}

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        serialize_slice(self)
    }
}

impl<T> Serialize for [T; 0] {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Array(Vec::new()))
    }
}

macro_rules! serialize_array {
    ( $len:expr ) => {
        impl<T> Serialize for [T; $len]
        where
            T: Serialize,
        {
            #[inline]
            fn serialize(&self) -> Result<Intermediate, Error> {
                serialize_slice(&self[..])
            }
        }
    };
}

serialize_array!(1);
serialize_array!(2);
serialize_array!(3);
serialize_array!(4);
serialize_array!(5);
serialize_array!(6);
serialize_array!(7);
serialize_array!(8);
serialize_array!(9);
serialize_array!(10);
serialize_array!(11);
serialize_array!(12);
serialize_array!(13);
serialize_array!(14);
serialize_array!(15);
serialize_array!(16);
serialize_array!(17);
serialize_array!(18);
serialize_array!(19);
serialize_array!(20);
serialize_array!(21);
serialize_array!(22);
serialize_array!(23);
serialize_array!(24);
serialize_array!(25);
serialize_array!(26);
serialize_array!(27);
serialize_array!(28);
serialize_array!(29);
serialize_array!(30);
serialize_array!(31);
serialize_array!(32);

impl Serialize for () {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(Intermediate::Array(Vec::new()))
    }
}

macro_rules! serialize_tuple {
    ( $len:expr => ($($n:tt $ty:ident)+) ) => {
        impl<$($ty),+> Serialize for ($($ty,)+)
        where
            $($ty: Serialize,)+
        {
            fn serialize(&self) -> Result<Intermediate, Error> {
                let res = vec![
                    $(
                        self.$n.serialize()?,
                    )+
                ];

                Ok(Intermediate::Array(res))
            }
        }
    };
}

serialize_tuple!(1 => (0 T0));
serialize_tuple!(2 => (0 T0 1 T1));
serialize_tuple!(3 => (0 T0 1 T1 2 T2));
serialize_tuple!(4 => (0 T0 1 T1 2 T2 3 T3));
serialize_tuple!(5 => (0 T0 1 T1 2 T2 3 T3 4 T4));
serialize_tuple!(6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5));
serialize_tuple!(7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6));
serialize_tuple!(8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7));
serialize_tuple!(9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8));
serialize_tuple!(10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9));
serialize_tuple!(11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10));
serialize_tuple!(12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11));
serialize_tuple!(13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12));
serialize_tuple!(14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13));
serialize_tuple!(15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14));
serialize_tuple!(16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15));

impl<K, V> Serialize for HashMap<K, V>
where
    K: ToString,
    V: Serialize,
{
    fn serialize(&self) -> Result<Intermediate, Error> {
        let mut res = Map::with_capacity(self.len());

        for (k, v) in self.iter() {
            res.insert_with_owned_key(k.to_string(), v.serialize()?);
        }

        Ok(Intermediate::Map(res))
    }
}

#[cfg(feature = "preserve-order")]
impl<K, V> Serialize for indexmap::IndexMap<K, V>
where
    K: ToString,
    V: Serialize,
{
    fn serialize(&self) -> Result<Intermediate, Error> {
        let mut res = Map::with_capacity(self.len());

        for (k, v) in self.iter() {
            res.insert_with_owned_key(k.to_string(), v.serialize()?);
        }

        Ok(Intermediate::Map(res))
    }
}

impl<'a, T> Serialize for &'a T
where
    T: Serialize + ?Sized,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        <T as Serialize>::serialize(self)
    }
}

impl<'a, T> Serialize for &'a mut T
where
    T: Serialize + ?Sized,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        <T as Serialize>::serialize(self)
    }
}

macro_rules! serialize_wrapper {
    ( $x:ident ) => {
        impl<T> Serialize for $x<T>
        where
            T: Serialize + ?Sized,
        {
            #[inline]
            fn serialize(&self) -> Result<Intermediate, Error> {
                <T as Serialize>::serialize(&*self)
            }
        }
        impl<T> Serialize for $x<[T]>
        where
            T: Serialize,
        {
            #[inline]
            fn serialize(&self) -> Result<Intermediate, Error> {
                <&[T] as Serialize>::serialize(&&**self)
            }
        }
    };
}

serialize_wrapper!(Box);
serialize_wrapper!(Rc);
serialize_wrapper!(Arc);

impl<T> Serialize for Mutex<T>
where
    T: Serialize + ?Sized,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        self.lock().unwrap().serialize()
    }
}

impl<T> Serialize for RefCell<T>
where
    T: Serialize + ?Sized,
{
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        self.borrow().serialize()
    }
}

/// Helper function.
fn serialize_slice<T>(v: &[T]) -> Result<Intermediate, Error>
where
    T: Serialize,
{
    let mut res = Vec::with_capacity(v.len());

    for elem in v.iter() {
        res.push(elem.serialize()?);
    }

    Ok(Intermediate::Array(res))
}

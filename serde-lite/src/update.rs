use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    ops::DerefMut,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{Deserialize, Error, Intermediate};

/// Update trait.
///
/// The trait can be used for objects that can be updated from the intermediate
/// representation.
pub trait Update: Deserialize {
    /// Update the object.
    fn update(&mut self, val: &Intermediate) -> Result<(), Error>;
}

macro_rules! update_by_replace {
    ( $x:ty ) => {
        impl Update for $x {
            #[inline]
            fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
                *self = <$x as Deserialize>::deserialize(val)?;

                Ok(())
            }
        }
    };
}

update_by_replace!(bool);

update_by_replace!(i8);
update_by_replace!(i16);
update_by_replace!(i32);
update_by_replace!(i64);
update_by_replace!(i128);
update_by_replace!(isize);

update_by_replace!(u8);
update_by_replace!(u16);
update_by_replace!(u32);
update_by_replace!(u64);
update_by_replace!(u128);
update_by_replace!(usize);

update_by_replace!(f32);
update_by_replace!(f64);

update_by_replace!(char);

update_by_replace!(String);

impl<T> Update for Option<T>
where
    T: Deserialize + Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        if val.is_none() {
            *self = None;
        } else if let Some(inner) = self {
            T::update(inner, val)?;
        } else {
            *self = T::deserialize(val).map(Some)?;
        }

        Ok(())
    }
}

impl<T> Update for Vec<T>
where
    T: Deserialize + Update,
{
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        if let Some(val) = val.as_array() {
            match val.len() {
                len if self.len() > len => self.truncate(len),
                len if self.len() < len => self.reserve(len - self.len()),
                _ => (),
            }

            for (index, elem) in val.iter().enumerate() {
                if let Some(current) = self.get_mut(index) {
                    current.update(elem)?;
                } else {
                    self.push(T::deserialize(elem)?);
                }
            }

            Ok(())
        } else {
            Err(Error::invalid_value_static("array"))
        }
    }
}

impl<T> Update for [T; 0] {
    #[inline]
    fn update(&mut self, _: &Intermediate) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! update_array {
    ( $len:expr => ($($n:tt)+) ) => {
        impl<T> Update for [T; $len]
        where
            T: Update,
        {
            fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
                if let Some(val) = val.as_array() {
                    if val.len() < $len {
                        return Err(Error::invalid_value_static(concat!(
                            "an array of length ",
                            $len
                        )));
                    }

                    for (index, elem) in val.iter().enumerate() {
                        self[index].update(elem)?;
                    }

                    Ok(())
                } else {
                    Err(Error::invalid_value_static(concat!(
                        "an array of length ",
                        $len
                    )))
                }
            }
        }
    };
}

update_array!(1 => (0));
update_array!(2 => (0 1));
update_array!(3 => (0 1 2));
update_array!(4 => (0 1 2 3));
update_array!(5 => (0 1 2 3 4));
update_array!(6 => (0 1 2 3 4 5));
update_array!(7 => (0 1 2 3 4 5 6));
update_array!(8 => (0 1 2 3 4 5 6 7));
update_array!(9 => (0 1 2 3 4 5 6 7 8));
update_array!(10 => (0 1 2 3 4 5 6 7 8 9));
update_array!(11 => (0 1 2 3 4 5 6 7 8 9 10));
update_array!(12 => (0 1 2 3 4 5 6 7 8 9 10 11));
update_array!(13 => (0 1 2 3 4 5 6 7 8 9 10 11 12));
update_array!(14 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13));
update_array!(15 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14));
update_array!(16 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15));
update_array!(17 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16));
update_array!(18 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17));
update_array!(19 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18));
update_array!(20 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19));
update_array!(21 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20));
update_array!(22 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21));
update_array!(23 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22));
update_array!(24 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23));
update_array!(25 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24));
update_array!(26 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25));
update_array!(27 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26));
update_array!(28 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27));
update_array!(29 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28));
update_array!(30 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29));
update_array!(31 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30));
update_array!(32 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31));

impl Update for () {
    #[inline]
    fn update(&mut self, _: &Intermediate) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! update_tuple {
    ( $len:expr => ($($n:tt $ty:ident)+) ) => {
        impl<$($ty),+> Update for ($($ty,)+)
        where
            $($ty: Update,)+
        {
            fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
                if let Some(val) = val.as_array() {
                    if val.len() < $len {
                        return Err(Error::invalid_value_static(concat!("an array of length ", $len)));
                    }

                    $(
                        self.$n.update(&val[$n])?;
                    )+

                    Ok(())
                } else {
                    Err(Error::invalid_value_static(concat!("an array of length ", $len)))
                }
            }
        }
    };
}

update_tuple!(1 => (0 T0));
update_tuple!(2 => (0 T0 1 T1));
update_tuple!(3 => (0 T0 1 T1 2 T2));
update_tuple!(4 => (0 T0 1 T1 2 T2 3 T3));
update_tuple!(5 => (0 T0 1 T1 2 T2 3 T3 4 T4));
update_tuple!(6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5));
update_tuple!(7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6));
update_tuple!(8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7));
update_tuple!(9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8));
update_tuple!(10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9));
update_tuple!(11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10));
update_tuple!(12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11));
update_tuple!(13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12));
update_tuple!(14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13));
update_tuple!(15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14));
update_tuple!(16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15));

impl<K, V> Update for HashMap<K, V>
where
    K: FromStr + Eq + Hash,
    K::Err: Display,
    V: Deserialize + Update,
{
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        let val = val
            .as_map()
            .ok_or_else(|| Error::invalid_value_static("map"))?;

        for (name, value) in val {
            let k = K::from_str(name).map_err(|err| Error::InvalidKey(err.to_string()))?;

            if let Some(inner) = self.get_mut(&k) {
                V::update(inner, value)?;
            } else {
                self.insert(k, V::deserialize(value)?);
            }
        }

        Ok(())
    }
}

#[cfg(feature = "preserve-order")]
impl<K, V> Update for indexmap::IndexMap<K, V>
where
    K: FromStr + Eq + Hash,
    K::Err: Display,
    V: Deserialize + Update,
{
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        let val = val
            .as_map()
            .ok_or_else(|| Error::invalid_value_static("map"))?;

        for (name, value) in val {
            let k = K::from_str(name).map_err(|err| Error::InvalidKey(err.to_string()))?;

            if let Some(inner) = self.get_mut(&k) {
                V::update(inner, value)?;
            } else {
                self.insert(k, V::deserialize(value)?);
            }
        }

        Ok(())
    }
}

impl<T> Update for Box<T>
where
    T: Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        self.deref_mut().update(val)
    }
}

impl<T> Update for Mutex<T>
where
    T: Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        self.get_mut().unwrap().update(val)
    }
}

impl<T> Update for Arc<Mutex<T>>
where
    T: Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        self.lock().unwrap().update(val)
    }
}

impl<T> Update for RefCell<T>
where
    T: Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        self.borrow_mut().update(val)
    }
}

impl<T> Update for Rc<RefCell<T>>
where
    T: Update,
{
    #[inline]
    fn update(&mut self, val: &Intermediate) -> Result<(), Error> {
        self.borrow_mut().update(val)
    }
}

use std::convert::TryFrom;

use crate::{atom::Atom, context::Context, result::Result, value::Value, Error, ErrorKind};

impl<'q> AsRef<Value<'q>> for Value<'q> {
    fn as_ref(&self) -> &Value<'q> {
        self
    }
}

pub trait IntoQj<'q> {
    fn into_qj(self, ctx: Context<'q>) -> Result<Value<'q>>;
}

impl<'q, T: Into<Value<'q>>> IntoQj<'q> for T {
    fn into_qj(self, _ctx: Context<'q>) -> Result<Value<'q>> {
        Ok(self.into())
    }
}

pub trait FromQj<'q>: Sized {
    fn from_qj(v: Value<'q>) -> Result<Self>;
}

impl<'q> FromQj<'q> for Value<'q> {
    fn from_qj(v: Value<'q>) -> Result<Self> {
        Ok(v)
    }
}

impl<'q, T> FromQj<'q> for T
where
    T: TryFrom<Value<'q>, Error = Error>,
{
    fn from_qj(v: Value<'q>) -> Result<Self> {
        T::try_from(v)
    }
}

pub trait FromQjMulti<'q, 'a>: Sized {
    fn from_qj_multi(v: &'a [Value<'q>]) -> Result<Self>;
}

impl<'q, 'a> FromQjMulti<'q, 'a> for &'a [Value<'q>] {
    fn from_qj_multi(v: &'a [Value<'q>]) -> Result<Self> {
        Ok(v)
    }
}

impl<'q, 'a> FromQjMulti<'q, 'a> for Vec<Value<'q>> {
    fn from_qj_multi(v: &[Value<'q>]) -> Result<Self> {
        Ok(v.to_vec())
    }
}

impl<'q, 'a> FromQjMulti<'q, 'a> for () {
    fn from_qj_multi(_v: &[Value<'q>]) -> Result<Self> {
        Ok(())
    }
}

macro_rules! impl_from_qj_multi_for_tuple {
    { for ($($k:expr => $t:ident),+) } => {
        impl<'q, 'a, $($t),+> FromQjMulti<'q, 'a> for ($($t,)+)
        where
            $($t: FromQj<'q>),+
        {
            fn from_qj_multi(v: &[Value<'q>]) -> Result<Self> {
                let err = |i: usize| move || Error::with_str(ErrorKind::RangeError, &format!("index: {}", i));
                Ok((
                    $($t::from_qj((v.get($k).ok_or_else(err($k))?.clone()))?,)+
                ))
            }
        }
    };
}

impl_from_qj_multi_for_tuple! { for (0 => T0) }
impl_from_qj_multi_for_tuple! { for (0 => T0, 1 => T1) }
impl_from_qj_multi_for_tuple! { for (0 => T0, 1 => T1, 2 => T2) }
impl_from_qj_multi_for_tuple! { for (0 => T0, 1 => T1, 2 => T2, 3 => T3) }

pub trait IntoQjMulti<'q> {
    type Target: AsRef<[Value<'q>]>;
    fn into_qj_multi(self, ctx: Context<'q>) -> Result<Self::Target>;
}

impl<'q> IntoQjMulti<'q> for &[Value<'q>] {
    type Target = Self;

    fn into_qj_multi(self, _ctx: Context<'q>) -> Result<Self::Target> {
        Ok(self)
    }
}

impl<'q, const N: usize> IntoQjMulti<'q> for &[Value<'q>; N] {
    type Target = Self;

    fn into_qj_multi(self, _ctx: Context<'q>) -> Result<Self::Target> {
        Ok(self)
    }
}

impl<'q, T: IntoQj<'q>> IntoQjMulti<'q> for Vec<T> {
    type Target = Vec<Value<'q>>;

    fn into_qj_multi(self, ctx: Context<'q>) -> Result<Self::Target> {
        let mut res = Vec::with_capacity(self.len());
        for v in self {
            res.push(v.into_qj(ctx)?);
        }
        Ok(res)
    }
}

macro_rules! impl_into_qj_multi_for_tuple {
    { for $size:expr => ($($k:tt => $t:ident),+) } => {
        impl<'q, $($t: IntoQj<'q>),+> IntoQjMulti<'q> for ($($t,)+) {
            type Target = Vec<Value<'q>>;

            fn into_qj_multi(self, ctx: Context<'q>) -> Result<Self::Target> {
                Ok(vec![$(self.$k.into_qj(ctx)?),+])
            }
        }
    };
}

impl<'q> IntoQjMulti<'q> for () {
    type Target = [Value<'q>; 0];

    fn into_qj_multi(self, _ctx: Context<'q>) -> Result<Self::Target> {
        Ok([])
    }
}

impl_into_qj_multi_for_tuple! { for 1 => (0 => T0) }
impl_into_qj_multi_for_tuple! { for 2 => (0 => T0, 1 => T1) }
impl_into_qj_multi_for_tuple! { for 3 => (0 => T0, 1 => T1, 2 => T2) }
impl_into_qj_multi_for_tuple! { for 4 => (0 => T0, 1 => T1, 2 => T2, 3 => T4) }

pub trait IntoQjAtom<'q> {
    fn into_qj_atom(self, ctx: Context<'q>) -> Result<Atom<'q>>;
}

impl<'q> IntoQjAtom<'q> for Atom<'q> {
    fn into_qj_atom(self, _ctx: Context<'q>) -> Result<Atom<'q>> {
        Ok(self)
    }
}

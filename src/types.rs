use qjncore::ValueTag;
use std::{
    any::type_name,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::instance::Data;
use std::convert::TryFrom;

macro_rules! impl_deref {
    { $target:ident for $type:ident } => {
        impl<'q> Deref for $type<'q> {
            type Target = $target<'q>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<'q> DerefMut for $type<'q> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

macro_rules! impl_from {
    { $source:ident for $type:ident } => {
        impl<'q> From<$source<'q>> for $type<'q> {
            fn from(v: $source<'q>) -> Self {
                unsafe { v.as_any::<Self>().clone() }
            }
        }
        impl<'q> AsRef<$type<'q>> for $source<'q> {
            fn as_ref(&self) -> &$type<'q> {
                self.as_data()
            }
        }
    };
    { $source:ident for $type:ty: |$v:pat| $implementation:expr } => {
        impl<'q> From<$source<'q>> for $type {
            fn from($v: $source<'q>) -> Self {
                $implementation
            }
        }
    };
}

macro_rules! impl_try_from {
    { $source:ident for $target:ident if $value:pat => $check:expr } => {
        impl<'q> TryFrom<$source<'q>> for $target<'q> {
            type Error = DataError;
            fn try_from(v: $source<'q>) -> Result<Self, Self::Error> {
                match &v {
                    $value if $check => Ok(unsafe { v.as_any::<Self>().clone() }),
                    _ => Err(DataError::bad_type::<$target, $source>())
                }
            }
        }
    };
}

// references
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Reference<'q>(Data<'q>);
impl_from! { Reference for Data }
impl_deref! { Data for Reference }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigDecimal<'q>(Reference<'q>);
impl_from! { BigDecimal for Data }
impl_try_from! { Data for BigDecimal if v => v.tag() == ValueTag::BigDecimal }
impl_deref! { Reference for BigDecimal }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigInt<'q>(Reference<'q>);
impl_from! { BigInt for Data }
impl_try_from! { Data for BigInt if v => v.tag() == ValueTag::BigInt }
impl_deref! { Reference for BigInt }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigFloat<'q>(Reference<'q>);
impl_from! { BigFloat for Data }
impl_try_from! { Data for BigFloat if v => v.tag() == ValueTag::BigFloat }
impl_deref! { Reference for BigFloat }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Symbol<'q>(Reference<'q>);
impl_from! { Symbol for Data }
impl_try_from! { Data for Symbol if v => v.tag() == ValueTag::Symbol }
impl_deref! { Reference for Symbol }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct String<'q>(Reference<'q>);
impl_from! { String for Data }
impl_try_from! { Data for String if v => v.tag() == ValueTag::String }
impl_deref! { Reference for String }

impl_from! { String for std::string::String: |v| v.to_string().unwrap() }

// pub struct Module<'q>(Data<'q>);

// pub struct FunctionBytecode<'q>(Data<'q>);

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Object<'q>(Reference<'q>);
impl_from! { Object for Data }
impl_try_from! { Data for Object if v => v.tag() == ValueTag::Object }
impl_deref! { Reference for Object }

// values
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Value<'q>(Data<'q>);
impl_from! { Value for Data }
impl_deref! { Data for Value }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Int<'q>(Value<'q>);
impl_from! { Int for Data }
impl_try_from! { Data for Int if v => v.tag() == ValueTag::Int }
impl_deref! { Value for Int }

impl_from!(Int for i32: |v| v.to_i32().unwrap());

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Bool<'q>(Value<'q>);
impl_from! { Bool for Data }
impl_try_from! { Data for Bool if v => v.tag() == ValueTag::Bool }
impl_deref! { Value for Bool }

impl_from! { Bool for bool: |v| v.to_bool().unwrap() }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Null<'q>(Value<'q>);
impl_from! { Null for Data }
impl_try_from! { Data for Null if v => v.tag() == ValueTag::Null }
impl_deref! { Value for Null }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Undefined<'q>(Value<'q>);
impl_from! { Undefined for Data }
impl_try_from! { Data for Undefined if v => v.tag() == ValueTag::Undefined }
impl_deref! { Value for Undefined }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct CatchOffset<'q>(Value<'q>);
impl_from! { CatchOffset for Data }
impl_try_from! { Data for CatchOffset if v => v.tag() == ValueTag::CatchOffset }
impl_deref! { Value for CatchOffset }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Exception<'q>(Value<'q>);
impl_from! { Exception for Data }
impl_try_from! { Data for Exception if v => v.tag() == ValueTag::Exception }
impl_deref! { Value for Exception }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Float64<'q>(Value<'q>);
impl_from! { Float64 for Data }
impl_try_from! { Data for Float64 if v => v.tag() == ValueTag::Float64 }
impl_deref! { Value for Float64 }

impl_from! { Float64 for f64: |v| v.to_f64().unwrap() }

#[derive(Clone, Copy, Debug)]
pub enum DataError {
    BadType {
        actual: &'static str,
        expected: &'static str,
    },
}

impl DataError {
    pub(crate) fn bad_type<E: 'static, A: 'static>() -> Self {
        Self::BadType {
            expected: type_name::<E>(),
            actual: type_name::<A>(),
        }
    }
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadType { expected, actual } => {
                write!(f, "expected type `{}`, got `{}`", expected, actual)
            }
        }
    }
}

impl Error for DataError {}

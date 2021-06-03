use crate::{
    convert::AsData,
    data::Data,
    error::{Error, ErrorKind},
};
pub use qjncore::ValueTag as Tag;
use std::{
    any::type_name,
    convert::TryFrom,
    fmt::{self, Formatter},
    ops::{Deref, DerefMut},
};

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

macro_rules! impl_as_data {
    { $source:ident } => {
        impl<'q> From<$source<'q>> for Data<'q> {
            fn from(v: $source<'q>) -> Self {
                unsafe { v.as_any::<Self>().clone() }
            }
        }
        impl<'q> AsData<'q> for $source<'q> {
            fn as_data(&self) -> &Data<'q> {
                self.as_data_raw()
            }
        }
        impl<'q> AsRef<Data<'q>> for $source<'q> {
            fn as_ref(&self) -> &Data<'q> {
                self.as_data()
            }
        }
    };
}

macro_rules! impl_from {
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
            type Error = Error;
            fn try_from(v: $source<'q>) -> Result<Self, Self::Error> {
                match &v {
                    $value if $check => Ok(unsafe { v.as_any::<Self>().clone() }),
                    _ => Err(Error::with_str(
                        ErrorKind::TypeError,
                        format!("can't convert {} to {}", type_name::<$source>(), type_name::<$target>()),
                    ))
                }
            }
        }
    };
}

// references
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Reference<'q>(Data<'q>);
impl_as_data! { Reference }
impl_deref! { Data for Reference }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigDecimal<'q>(Reference<'q>);
impl_as_data! { BigDecimal }
impl_try_from! { Data for BigDecimal if v => v.tag() == Tag::BigDecimal }
impl_deref! { Reference for BigDecimal }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigInt<'q>(Reference<'q>);
impl_as_data! { BigInt }
impl_try_from! { Data for BigInt if v => v.tag() == Tag::BigInt }
impl_deref! { Reference for BigInt }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigFloat<'q>(Reference<'q>);
impl_as_data! { BigFloat }
impl_try_from! { Data for BigFloat if v => v.tag() == Tag::BigFloat }
impl_deref! { Reference for BigFloat }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Symbol<'q>(Reference<'q>);
impl_as_data! { Symbol }
impl_try_from! { Data for Symbol if v => v.tag() == Tag::Symbol }
impl_deref! { Reference for Symbol }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct String<'q>(Reference<'q>);
impl_as_data! { String }
impl_try_from! { Data for String if v => v.tag() == Tag::String }
impl_deref! { Reference for String }

impl_from! { String for std::string::String: |v| v.to_string().unwrap() }

// pub struct Module<'q>(Data<'q>);

// pub struct FunctionBytecode<'q>(Data<'q>);

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Object<'q>(Reference<'q>);
impl_as_data! { Object }
impl_try_from! { Data for Object if v => v.tag() == Tag::Object }
impl_deref! { Reference for Object }

// values
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Value<'q>(Data<'q>);
impl_as_data! { Value }
impl_deref! { Data for Value }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Int<'q>(Value<'q>);
impl_as_data! { Int }
impl_try_from! { Data for Int if v => v.tag() == Tag::Int }
impl_deref! { Value for Int }

impl_from!(Int for i32: |v| v.to_i32().unwrap());

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Bool<'q>(Value<'q>);
impl_as_data! { Bool }
impl_try_from! { Data for Bool if v => v.tag() == Tag::Bool }
impl_deref! { Value for Bool }

impl_from! { Bool for bool: |v| v.to_bool().unwrap() }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Null<'q>(Value<'q>);
impl_as_data! { Null }
impl_try_from! { Data for Null if v => v.tag() == Tag::Null }
impl_deref! { Value for Null }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Undefined<'q>(Value<'q>);
impl_as_data! { Undefined }
impl_try_from! { Data for Undefined if v => v.tag() == Tag::Undefined }
impl_deref! { Value for Undefined }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Uninitialized<'q>(Value<'q>);
impl_as_data! { Uninitialized }
impl_try_from! { Data for Uninitialized if v => v.tag() == Tag::Uninitialized }
impl_deref! { Value for Uninitialized }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct CatchOffset<'q>(Value<'q>);
impl_as_data! { CatchOffset }
impl_try_from! { Data for CatchOffset if v => v.tag() == Tag::CatchOffset }
impl_deref! { Value for CatchOffset }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Exception<'q>(Value<'q>);
impl_as_data! { Exception }
impl_try_from! { Data for Exception if v => v.tag() == Tag::Exception }
impl_deref! { Value for Exception }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Float64<'q>(Value<'q>);
impl_as_data! { Float64 }
impl_try_from! { Data for Float64 if v => v.tag() == Tag::Float64 }
impl_deref! { Value for Float64 }

impl_from! { Float64 for f64: |v| v.to_f64().unwrap() }

#[non_exhaustive]
pub enum Variant<'q> {
    BigDecimal(BigDecimal<'q>),
    BigInt(BigInt<'q>),
    BigFloat(BigFloat<'q>),
    Symbol(Symbol<'q>),
    String(String<'q>),
    Object(Object<'q>),
    Int(i32),
    Bool(bool),
    Null,
    Undefined,
    Uninitialized,
    CatchOffset,
    Exception,
    Float64(f64),
}

impl<'q> fmt::Debug for Variant<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Variant::BigDecimal(v) => f.write_str(format!("BigDecimal({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::BigInt(v) => f.write_str(format!("BigInt({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::BigFloat(v) => f.write_str(format!("BigFloat({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::Symbol(v) => f.write_str(format!("Symbol({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::String(v) => f.write_str(format!("String({:?})", v.to_string().unwrap()).as_str()),
            Variant::Object(v) => f.write_str(format!("Object({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::Int(v) => f.write_str(format!("Int({})", v).as_str()),
            Variant::Bool(v) => f.write_str(format!("Bool({})", v).as_str()),
            Variant::Null => f.write_str("Null"),
            Variant::Undefined => f.write_str("Undefined"),
            Variant::Uninitialized => f.write_str("Uninitialized"),
            Variant::CatchOffset => f.write_str("CatchOffset"),
            Variant::Exception => f.write_str("Exception"),
            Variant::Float64(v) => f.write_str(format!("Float64({})", v).as_str()),
            #[allow(unreachable_patterns)]
            _ => f.write_str("Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{run_with_context, Bool, EvalFlags, Float64, Int, Result, String as QjString, Variant};
    use std::convert::TryInto;

    macro_rules! assert_match {
        ( $expect:pat => $ret:expr, $actual:expr ) => {
            match $actual {
                $expect => $ret,
                _ => {
                    assert!(false, "`{}` doesn't match to the pattern `{}`", stringify!($actual), stringify!($expect));
                    panic!()
                },
            }
        };
        ( $expect:pat, $actual:expr ) => {
            assert_match!($expect => (), $actual)
        }
    }

    #[test]
    fn test() -> Result<()> {
        run_with_context(|ctx| {
            let v = ctx.eval("2n ** 128n", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::BigInt(_), v.to_variant());
            assert_eq!("340282366920938463463374607431768211456", v.to_string()?);

            let v = ctx.eval("Symbol('foo')", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Symbol(_), v.to_variant());

            let v = ctx.eval("\"foo\"", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::String(_), v.to_variant());
            let s: QjString = v.try_into()?;
            let s: String = s.into();
            assert_eq!("foo", s);

            let v = ctx.eval("({})", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Object(_), v.to_variant());

            let v = ctx.eval("() => {}", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Object(_), v.to_variant());

            let v = ctx.eval("42", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Int(42), v.to_variant());
            let i: Int = v.try_into()?;
            let i: i32 = i.into();
            assert_eq!(42, i);

            let v = ctx.eval("true", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Bool(true), v.to_variant());
            let b: Bool = v.try_into()?;
            let b: bool = b.into();
            assert_eq!(true, b);

            let v = ctx.eval("null", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Null, v.to_variant());

            let v = ctx.eval("void 0", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Undefined, v.to_variant());

            let v = ctx.eval("0.25", "<input>", EvalFlags::TYPE_GLOBAL)?;
            let f = assert_match!(Variant::Float64(f) => f, v.to_variant());
            assert_eq!(0.25, f);
            let f: Float64 = v.try_into()?;
            let f: f64 = f.into();
            assert_eq!(0.25, f);

            Ok(())
        })
    }
}

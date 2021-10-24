use crate::{
    class::Class,
    context::Context,
    convert::IntoQj,
    error::{Error, ErrorKind},
    result::Result,
    util::Opaque,
    value::Value,
};
pub use quijine_core::ValueTag as Tag;
use std::{
    any::type_name,
    convert::TryFrom,
    fmt::{self, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    result::Result as StdResult,
    string::String as StdString,
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

macro_rules! impl_as_ref_value {
    { for $source:ident } => {
        impl<'q> From<$source<'q>> for Value<'q> {
            fn from(v: $source<'q>) -> Self {
                unsafe { v.as_any::<Self>().clone() }
            }
        }
        impl<'q> AsRef<Value<'q>> for $source<'q> {
            fn as_ref(&self) -> &Value<'q> {
                self.as_value_raw()
            }
        }
    };
}

macro_rules! impl_from {
    { $source:ident for $type:ty: |$v:pat_param| $implementation:expr } => {
        impl<'q> From<$source<'q>> for $type {
            fn from($v: $source<'q>) -> Self {
                $implementation
            }
        }
    };
}

macro_rules! impl_try_from {
    { $source:ident for $type:ty: |$v:pat_param| $implementation:expr } => {
        impl<'q> TryFrom<$source<'q>> for $type {
            type Error = Error;
            fn try_from($v: $source<'q>) -> Result<Self> {
                $implementation
            }
        }
    };
}

macro_rules! impl_into_qj {
    { for $type:ty: |$v: pat_param, $ctx:pat_param| $implementation:expr } => {
        impl<'q> IntoQj<'q> for $type {
            fn into_qj(self, $ctx: Context<'q>) -> Result<Value<'q>> {
                let $v = self;
                $implementation
            }
        }
    };
}

macro_rules! impl_try_from_value {
    { $source:ident for $target:ident if $value:pat => $check:expr } => {
        impl<'q> TryFrom<$source<'q>> for $target<'q> {
            type Error = Error;
            fn try_from(v: $source<'q>) -> StdResult<Self, Self::Error> {
                match &v {
                    $value if $check => Ok(unsafe { v.as_any::<Self>().clone() }),
                    _ => Err(Error::with_str(
                        ErrorKind::TypeError,
                        &format!("can't convert {} to {}", type_name::<$source>(), type_name::<$target>()),
                    ))
                }
            }
        }
    };
}

// references
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct HasPtr<'q>(Value<'q>);
impl_as_ref_value! { for HasPtr }
impl_deref! { Value for HasPtr }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigDecimal<'q>(HasPtr<'q>);
impl_as_ref_value! { for BigDecimal }
impl_try_from_value! { Value for BigDecimal if v => v.tag() == Tag::BigDecimal }
impl_deref! { HasPtr for BigDecimal }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigInt<'q>(HasPtr<'q>);
impl_as_ref_value! { for BigInt }
impl_try_from_value! { Value for BigInt if v => v.tag() == Tag::BigInt }
impl_deref! { HasPtr for BigInt }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct BigFloat<'q>(HasPtr<'q>);
impl_as_ref_value! { for BigFloat }
impl_try_from_value! { Value for BigFloat if v => v.tag() == Tag::BigFloat }
impl_deref! { HasPtr for BigFloat }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Symbol<'q>(HasPtr<'q>);
impl_as_ref_value! { for Symbol }
impl_try_from_value! { Value for Symbol if v => v.tag() == Tag::Symbol }
impl_deref! { HasPtr for Symbol }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct String<'q>(HasPtr<'q>);
impl_as_ref_value! { for String }
impl_try_from_value! { Value for String if v => v.tag() == Tag::String }
impl_deref! { HasPtr for String }

impl_from! { String for StdString: |v| v.to_string().unwrap() }
impl_try_from! { Value for StdString: |v| v.to_string() }
impl_into_qj! { for &str: |v, ctx| ctx.new_string(v).map(|v| v.into()) }
impl_into_qj! { for StdString: |v, ctx| ctx.new_string(&v).map(|v| v.into()) }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Module<'q>(HasPtr<'q>);
impl_as_ref_value! { for Module }
impl_try_from_value! { Value for Module if v => v.tag() == Tag::Module }
impl_deref! { HasPtr for Module }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct FunctionBytecode<'q>(HasPtr<'q>);
impl_as_ref_value! { for FunctionBytecode }
impl_try_from_value! { Value for FunctionBytecode if v => v.tag() == Tag::FunctionBytecode }
impl_deref! { HasPtr for FunctionBytecode }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Object<'q>(HasPtr<'q>);
impl_as_ref_value! { for Object }
impl_try_from_value! { Value for Object if v => v.tag() == Tag::Object }
impl_deref! { HasPtr for Object }

// values
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct HasVal<'q>(Value<'q>);
impl_as_ref_value! { for HasVal }
impl_deref! { Value for HasVal }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Int<'q>(HasVal<'q>);
impl_as_ref_value! { for Int }
impl_try_from_value! { Value for Int if v => v.tag() == Tag::Int }
impl_deref! { HasVal for Int }

impl_from!(Int for i32: |v| v.to_i32().unwrap());
impl_try_from! { Value for i32: |v| v.to_i32() }
impl_into_qj! { for i32: |v, ctx| Ok(ctx.new_int32(v).into()) }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Bool<'q>(HasVal<'q>);
impl_as_ref_value! { for Bool }
impl_try_from_value! { Value for Bool if v => v.tag() == Tag::Bool }
impl_deref! { HasVal for Bool }

impl_from! { Bool for bool: |v| v.to_bool().unwrap() }
impl_try_from! { Value for bool: |v| v.to_bool() }
impl_into_qj! { for bool: |v, ctx| Ok(ctx.new_bool(v).into()) }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Null<'q>(HasVal<'q>);
impl_as_ref_value! { for Null }
impl_try_from_value! { Value for Null if v => v.tag() == Tag::Null }
impl_deref! { HasVal for Null }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Undefined<'q>(HasVal<'q>);
impl_as_ref_value! { for Undefined }
impl_try_from_value! { Value for Undefined if v => v.tag() == Tag::Undefined }
impl_deref! { HasVal for Undefined }

impl_from! { Undefined for (): |_v| () }
// this cause unexpected type conversion: v.set("bar", v.get("foo")?)?;
// impl_try_from! { Value for (): |v| Ok(v.context().undefined().into()) }
impl_into_qj! { for (): |_v, ctx| Ok(ctx.undefined().into()) }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Uninitialized<'q>(HasVal<'q>);
impl_as_ref_value! { for Uninitialized }
impl_try_from_value! { Value for Uninitialized if v => v.tag() == Tag::Uninitialized }
impl_deref! { HasVal for Uninitialized }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct CatchOffset<'q>(HasVal<'q>);
impl_as_ref_value! { for CatchOffset }
impl_try_from_value! { Value for CatchOffset if v => v.tag() == Tag::CatchOffset }
impl_deref! { HasVal for CatchOffset }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Exception<'q>(HasVal<'q>);
impl_as_ref_value! { for Exception }
impl_try_from_value! { Value for Exception if v => v.tag() == Tag::Exception }
impl_deref! { HasVal for Exception }

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Float64<'q>(HasVal<'q>);
impl_as_ref_value! { for Float64 }
impl_try_from_value! { Value for Float64 if v => v.tag() == Tag::Float64 }
impl_deref! { HasVal for Float64 }

impl_from! { Float64 for f64: |v| v.to_f64().unwrap() }
impl_try_from! { Value for f64: |v| v.to_f64() }
impl_into_qj! { for f64: |v, ctx| Ok(ctx.new_float64(v).into()) }

#[non_exhaustive]
pub enum Variant<'q> {
    BigDecimal(BigDecimal<'q>),
    BigInt(BigInt<'q>),
    BigFloat(BigFloat<'q>),
    Symbol(Symbol<'q>),
    String(String<'q>),
    Module(Module<'q>),
    FunctionBytecode(FunctionBytecode<'q>),
    Object(Object<'q>),
    Int(i32),
    Bool(bool),
    Null,
    Undefined,
    Uninitialized,
    CatchOffset(Opaque<4>),
    Exception,
    Float64(f64),
}

fn format_reference<'q, T: AsRef<Value<'q>>>(name: &str, v: &T) -> StdString {
    format!(
        "{}({:p}: {})",
        name,
        v.as_ref().to_ptr().unwrap(),
        v.as_ref().to_string().unwrap()
    )
}

impl<'q> fmt::Debug for Variant<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Variant::BigDecimal(v) => f.write_str(format_reference("BigDecimal", v).as_str()),
            Variant::BigInt(v) => f.write_str(format_reference("BigInt", v).as_str()),
            Variant::BigFloat(v) => f.write_str(format_reference("BigFloat", v).as_str()),
            Variant::Symbol(v) => f.write_str(format!("Symbol({:p})", v.to_ptr().unwrap()).as_str()),
            Variant::String(v) => f.write_str(format_reference("String", v).as_str()),
            Variant::Module(v) => f.write_str(format_reference("Module", v).as_str()),
            Variant::FunctionBytecode(v) => f.write_str(format_reference("FunctionBytecode", v).as_str()),
            Variant::Object(v) => f.write_str(format_reference("Object", v).as_str()),
            Variant::Int(v) => f.write_str(format!("Int({})", v).as_str()),
            Variant::Bool(v) => f.write_str(format!("Bool({})", v).as_str()),
            Variant::Null => f.write_str("Null"),
            Variant::Undefined => f.write_str("Undefined"),
            Variant::Uninitialized => f.write_str("Uninitialized"),
            Variant::CatchOffset(_) => f.write_str("CatchOffset(_)"),
            Variant::Exception => f.write_str("Exception"),
            Variant::Float64(v) => f.write_str(format!("Float64({})", v).as_str()),
            #[allow(unreachable_patterns)]
            _ => f.write_str("Unknown"),
        }
    }
}

// class object

#[derive(Debug)]
#[repr(transparent)]
pub struct ClassObject<'q, C: Class + 'static>(Object<'q>, PhantomData<C>);

impl<'q, C: Class + 'static> Clone for ClassObject<'q, C> {
    fn clone(&self) -> Self {
        ClassObject(self.0.clone(), PhantomData)
    }
}

impl<'q, C: Class + 'static> From<ClassObject<'q, C>> for Value<'q> {
    fn from(v: ClassObject<'q, C>) -> Self {
        unsafe { v.as_any::<Self>().clone() }
    }
}

impl<'q, C: Class + 'static> AsRef<Value<'q>> for ClassObject<'q, C> {
    fn as_ref(&self) -> &Value<'q> {
        self.as_value_raw()
    }
}

impl<'q, C: Class + 'static> TryFrom<Value<'q>> for ClassObject<'q, C> {
    type Error = Error;

    fn try_from(v: Value<'q>) -> StdResult<Self, Self::Error> {
        match &v {
            v if v.tag() == Tag::Object && v.opaque::<C>().is_some() => Ok(unsafe { v.as_any::<Self>().clone() }),
            _ => Err(Error::with_str(
                ErrorKind::TypeError,
                &format!(
                    "can't convert {} to {}",
                    type_name::<Value>(),
                    type_name::<ClassObject<C>>()
                ),
            )),
        }
    }
}

impl<'q, C: Class + 'static> Deref for ClassObject<'q, C> {
    type Target = Object<'q>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'q, C: Class + 'static> DerefMut for ClassObject<'q, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{context, Bool, EvalFlags, Float64, Int, Result, String as QjString, Value, Variant};
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
        context(|ctx| {
            let v: Value = ctx.eval("2n ** 128n", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::BigInt(_), v.to_variant());
            assert_eq!("340282366920938463463374607431768211456", v.to_string()?);

            let v: Value = ctx.eval("Symbol('foo')", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Symbol(_), v.to_variant());

            let v: Value = ctx.eval("\"foo\"", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::String(_), v.to_variant());
            let s: QjString = v.try_into()?;
            let s: String = s.into();
            assert_eq!("foo", s);

            let v: Value = ctx.eval("42", "<input>", EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY)?;
            assert_match!(Variant::FunctionBytecode(_), v.to_variant());

            let v: Value = ctx.eval("({})", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Object(_), v.to_variant());

            let v: Value = ctx.eval("() => {}", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Object(_), v.to_variant());

            let v: Value = ctx.eval("[2, 3, 5, 7]", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Object(_), v.to_variant());

            let v: Value = ctx.eval("42", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Int(42), v.to_variant());
            let i: Int = v.try_into()?;
            let i: i32 = i.into();
            assert_eq!(42, i);

            let v: Value = ctx.eval("true", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Bool(true), v.to_variant());
            let b: Bool = v.try_into()?;
            let b: bool = b.into();
            assert_eq!(true, b);

            let v: Value = ctx.eval("null", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Null, v.to_variant());

            let v: Value = ctx.eval("void 0", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_match!(Variant::Undefined, v.to_variant());

            let v: Value = ctx.eval("0.25", "<input>", EvalFlags::TYPE_GLOBAL)?;
            let f = assert_match!(Variant::Float64(f) => f, v.to_variant());
            assert_eq!(0.25, f);
            let f: Float64 = v.try_into()?;
            let f: f64 = f.into();
            assert_eq!(0.25, f);

            Ok(())
        })
    }
}

use crate::instance::Qj;
use std::{fmt, fmt::Formatter};

// any
pub struct QjAnyTag;

// references
pub struct QjReferenceTag;

pub struct QjBigDecimalTag;
pub struct QjBigIntTag;
pub struct QjBigFloatTag;
pub struct QjSymbolTag;
pub struct QjStringTag;
// pub struct QjModuleTag; // used internally
// pub struct QjFunctionBytecodeTag; // used internally
pub struct QjObjectTag;

// values
pub struct QjValueTag;

pub struct QjIntTag;
pub struct QjBoolTag;
pub struct QjNullTag;
pub struct QjUndefinedTag;
pub struct QjUninitializedTag;
pub struct QjCatchOffsetTag;
pub struct QjExceptionTag;
pub struct QjFloat64Tag;

#[non_exhaustive]
pub enum QjVariant<'q> {
    BigDecimal(Qj<'q, QjBigDecimalTag>),
    BigInt(Qj<'q, QjBigIntTag>),
    BigFloat(Qj<'q, QjBigFloatTag>),
    Symbol(Qj<'q, QjSymbolTag>),
    String(Qj<'q, QjStringTag>),
    Object(Qj<'q, QjObjectTag>),
    Int(i32),
    Bool(bool),
    Null,
    Undefined,
    Uninitialized,
    CatchOffset,
    Exception,
    Float64(f64),
}

impl<'q> fmt::Debug for QjVariant<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            QjVariant::BigDecimal(v) => f.write_str(format!("BigDecimal({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::BigInt(v) => f.write_str(format!("BigInt({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::BigFloat(v) => f.write_str(format!("BigFloat({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::Symbol(v) => f.write_str(format!("Symbol({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::String(v) => f.write_str(format!("String({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::Object(v) => f.write_str(format!("Object({:p})", v.to_ptr().unwrap()).as_str()),
            QjVariant::Int(v) => f.write_str(format!("Int({})", v).as_str()),
            QjVariant::Bool(v) => f.write_str(format!("Bool({})", v).as_str()),
            QjVariant::Null => f.write_str("Null"),
            QjVariant::Undefined => f.write_str("Undefined"),
            QjVariant::Uninitialized => f.write_str("Uninitialized"),
            QjVariant::CatchOffset => f.write_str("CatchOffset"),
            QjVariant::Exception => f.write_str("Exception"),
            QjVariant::Float64(v) => f.write_str(format!("Float64({})", v).as_str()),
            #[allow(unreachable_patterns)]
            _ => f.write_str("Unknown"),
        }
    }
}

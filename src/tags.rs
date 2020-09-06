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
            QjVariant::BigDecimal(_) => f.write_str("BigDecimal(_)"),
            QjVariant::BigInt(_) => f.write_str("BigInt(_)"),
            QjVariant::BigFloat(_) => f.write_str("BigFloat(_)"),
            QjVariant::Symbol(_) => f.write_str("Symbol(_)"),
            QjVariant::String(_) => f.write_str("String(_)"),
            QjVariant::Object(_) => f.write_str("Object(_)"),
            QjVariant::Int(v) => f.write_str(format!("Int({})", v).as_str()),
            QjVariant::Bool(v) => f.write_str(format!("Bool({})", v).as_str()),
            QjVariant::Null => f.write_str("Null"),
            QjVariant::Undefined => f.write_str("Undefined"),
            QjVariant::Uninitialized => f.write_str("Uninitialized"),
            QjVariant::CatchOffset => f.write_str("CatchOffset"),
            QjVariant::Exception => f.write_str("Exception"),
            QjVariant::Float64(v) => f.write_str(format!("Float64({})", v).as_str()),
            _ => f.write_str("Unknown"),
        }
    }
}

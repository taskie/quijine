use crate::{instance::Qj};

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

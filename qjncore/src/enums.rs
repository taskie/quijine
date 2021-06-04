#![allow(dead_code)]

use crate::ffi;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CFunctionEnum {
    Generic = ffi::JSCFunctionEnum_JS_CFUNC_generic,
    GenericMagic = ffi::JSCFunctionEnum_JS_CFUNC_generic_magic,
    Constructor = ffi::JSCFunctionEnum_JS_CFUNC_constructor,
    ConstructorMagic = ffi::JSCFunctionEnum_JS_CFUNC_constructor_magic,
    ConstructorOrFunc = ffi::JSCFunctionEnum_JS_CFUNC_constructor_or_func,
    ConstructorOrFuncMagic = ffi::JSCFunctionEnum_JS_CFUNC_constructor_or_func_magic,
    FF = ffi::JSCFunctionEnum_JS_CFUNC_f_f,
    #[allow(clippy::upper_case_acronyms)]
    FFF = ffi::JSCFunctionEnum_JS_CFUNC_f_f_f,
    Getter = ffi::JSCFunctionEnum_JS_CFUNC_getter,
    Setter = ffi::JSCFunctionEnum_JS_CFUNC_setter,
    GetterMagic = ffi::JSCFunctionEnum_JS_CFUNC_getter_magic,
    SetterMagic = ffi::JSCFunctionEnum_JS_CFUNC_setter_magic,
    IteratorNext = ffi::JSCFunctionEnum_JS_CFUNC_iterator_next,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ValueTag {
    BigDecimal = ffi::JS_TAG_BIG_DECIMAL,
    BigInt = ffi::JS_TAG_BIG_INT,
    BigFloat = ffi::JS_TAG_BIG_FLOAT,
    Symbol = ffi::JS_TAG_SYMBOL,
    String = ffi::JS_TAG_STRING,
    Module = ffi::JS_TAG_MODULE,
    FunctionBytecode = ffi::JS_TAG_FUNCTION_BYTECODE,
    Object = ffi::JS_TAG_OBJECT,
    Int = ffi::JS_TAG_INT,
    Bool = ffi::JS_TAG_BOOL,
    Null = ffi::JS_TAG_NULL,
    Undefined = ffi::JS_TAG_UNDEFINED,
    Uninitialized = ffi::JS_TAG_UNINITIALIZED,
    CatchOffset = ffi::JS_TAG_CATCH_OFFSET,
    Exception = ffi::JS_TAG_EXCEPTION,
    Float64 = ffi::JS_TAG_FLOAT64,
}

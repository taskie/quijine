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
    FFF = ffi::JSCFunctionEnum_JS_CFUNC_f_f_f,
    Getter = ffi::JSCFunctionEnum_JS_CFUNC_getter,
    Setter = ffi::JSCFunctionEnum_JS_CFUNC_setter,
    GetterMagic = ffi::JSCFunctionEnum_JS_CFUNC_getter_magic,
    SetterMagic = ffi::JSCFunctionEnum_JS_CFUNC_setter_magic,
    IteratorNext = ffi::JSCFunctionEnum_JS_CFUNC_iterator_next,
}

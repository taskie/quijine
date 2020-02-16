pub use libquickjs_sys::*;

use std::{
    os::raw::{c_char, c_int, c_void},
    ptr::null_mut,
};

// from C preprocessor macro

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_VALUE_GET_TAG(v: JSValue) -> i32 {
    v.tag as i32
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_VALUE_GET_PTR(v: JSValue) -> *mut c_void {
    v.u.ptr
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_VALUE_HAS_REF_COUNT(v: JSValue) -> bool {
    JS_VALUE_GET_TAG(v) as u32 >= JS_TAG_FIRST as u32
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_FreeValue(ctx: *mut JSContext, v: JSValue) {
    if JS_VALUE_HAS_REF_COUNT(v) {
        let p = JS_VALUE_GET_PTR(v) as *mut JSRefCountHeader;
        let pref: &mut JSRefCountHeader = &mut *p;
        pref.ref_count -= 1;
        if pref.ref_count <= 0 {
            __JS_FreeValue(ctx, v);
        }
    }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_DupValue(_ctx: *mut JSContext, v: JSValue) -> JSValue {
    if JS_VALUE_HAS_REF_COUNT(v) {
        let p = JS_VALUE_GET_PTR(v) as *mut JSRefCountHeader;
        let pref: &mut JSRefCountHeader = &mut *p;
        pref.ref_count += 1;
    }
    v
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_FreeValueRT(rt: *mut JSRuntime, v: JSValue) {
    if JS_VALUE_HAS_REF_COUNT(v) {
        let p = JS_VALUE_GET_PTR(v) as *mut JSRefCountHeader;
        let pref: &mut JSRefCountHeader = &mut *p;
        pref.ref_count -= 1;
        if pref.ref_count <= 0 {
            __JS_FreeValueRT(rt, v);
        }
    }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_DupValueRT(_rt: *mut JSRuntime, v: JSValue) -> JSValue {
    if JS_VALUE_HAS_REF_COUNT(v) {
        let p = JS_VALUE_GET_PTR(v) as *mut JSRefCountHeader;
        let pref: &mut JSRefCountHeader = &mut *p;
        pref.ref_count += 1;
    }
    v
}

#[cold]
fn colder<T>(x: T) -> T {
    x
}

pub(crate) fn builtin_expect<T>(x: T, y: T) -> T
where
    T: PartialEq,
{
    if x == y {
        x
    } else {
        colder(x)
    }
}

#[inline]
pub(crate) fn js_unlikely(x: bool) -> bool {
    builtin_expect(x, false)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsNull(v: JSValue) -> bool {
    JS_VALUE_GET_TAG(v) == JS_TAG_NULL
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsException(v: JSValue) -> bool {
    js_unlikely(JS_VALUE_GET_TAG(v) == JS_TAG_EXCEPTION)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsUndefined(v: JSValue) -> bool {
    JS_VALUE_GET_TAG(v) == JS_TAG_UNDEFINED
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsUninitialized(v: JSValue) -> bool {
    JS_VALUE_GET_TAG(v) == JS_TAG_UNINITIALIZED
}

macro_rules! JS_MKVAL {
    ($tag: expr, $val: expr) => {
        JSValue {
            u: JSValueUnion { int32: $val as i32 },
            tag: $tag as i64,
        }
    };
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewInt32(_ctx: *mut JSContext, val: i32) -> JSValue {
    JS_MKVAL!(JS_TAG_INT, val)
}

pub const JS_NULL: JSValue = JS_MKVAL!(JS_TAG_NULL, 0);
pub const JS_FALSE: JSValue = JS_MKVAL!(JS_TAG_BOOL, 0);
pub const JS_TRUE: JSValue = JS_MKVAL!(JS_TAG_BOOL, 1);
pub const JS_UNDEFINED: JSValue = JS_MKVAL!(JS_TAG_UNDEFINED, 0);
pub const JS_EXCEPTION: JSValue = JS_MKVAL!(JS_TAG_EXCEPTION, 0);
pub const JS_UNINITIALIZED: JSValue = JS_MKVAL!(JS_TAG_UNINITIALIZED, 0);

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ToCString(ctx: *mut JSContext, val1: JSValue) -> *const c_char {
    JS_ToCStringLen2(ctx, null_mut(), val1, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewCFunction(ctx: *mut JSContext, func: JSCFunction, name: *const c_char, length: c_int) -> JSValue {
    JS_NewCFunction2(ctx, func, name, length, JSCFunctionEnum_JS_CFUNC_generic, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_GetProperty(ctx: *mut JSContext, this_obj: JSValue, prop: JSAtom) -> JSValue {
    JS_GetPropertyInternal(ctx, this_obj, prop, this_obj, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_SetProperty(ctx: *mut JSContext, this_obj: JSValue, prop: JSAtom, _val: JSValue) -> c_int {
    JS_SetPropertyInternal(ctx, this_obj, prop, this_obj, JS_PROP_THROW as i32)
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CFunctionEnum {
    Generic = JSCFunctionEnum_JS_CFUNC_generic,
    GenericMagic = JSCFunctionEnum_JS_CFUNC_generic_magic,
    Constructor = JSCFunctionEnum_JS_CFUNC_constructor,
    ConstructorMagic = JSCFunctionEnum_JS_CFUNC_constructor_magic,
    ConstructorOrFunc = JSCFunctionEnum_JS_CFUNC_constructor_or_func,
    ConstructorOrFuncMagic = JSCFunctionEnum_JS_CFUNC_constructor_or_func_magic,
    FF = JSCFunctionEnum_JS_CFUNC_f_f,
    FFF = JSCFunctionEnum_JS_CFUNC_f_f_f,
    Getter = JSCFunctionEnum_JS_CFUNC_getter,
    Setter = JSCFunctionEnum_JS_CFUNC_setter,
    GetterMagic = JSCFunctionEnum_JS_CFUNC_getter_magic,
    SetterMagic = JSCFunctionEnum_JS_CFUNC_setter_magic,
    IteratorNext = JSCFunctionEnum_JS_CFUNC_iterator_next,
}

macro_rules! JS_CFUNC_INTERNAL_DEF {
    ($name: expr, $length: expr, $cproto: expr, $field: ident, $func1: expr, $magic: expr) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: (JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE) as u8,
            def_type: JS_DEF_CFUNC as u8,
            magic: $magic as i16,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                func: JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_1 {
                    length: $length as u8,
                    cproto: $cproto as u8,
                    cfunc: JSCFunctionType { $field: $func1 },
                },
            },
        }
    };
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_DEF(name: *const c_char, length: u8, func1: JSCFunction) -> JSCFunctionListEntry {
    JS_CFUNC_INTERNAL_DEF!(name, length, CFunctionEnum::Generic, generic, func1, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_SPECIAL_DEF_constructor(
    name: *const c_char,
    length: u8,
    func1: JSCFunction,
) -> JSCFunctionListEntry {
    JS_CFUNC_INTERNAL_DEF!(name, length, CFunctionEnum::Constructor, constructor, func1, 0)
}

type JSCFunctionRaw =
    unsafe extern "C" fn(ctx: *mut JSContext, this_val: JSValue, argc: c_int, argv: *mut JSValue) -> JSValue;

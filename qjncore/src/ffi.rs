#![allow(clippy::missing_safety_doc)]

pub use libquickjs_sys::*;
use std::{
    mem::transmute,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr::null_mut,
};

// private type definitions

#[cfg(any(target_pointer_width = "32"))]
#[allow(non_camel_case_types)]
type c_size_t = u32;
#[cfg(any(target_pointer_width = "64"))]
#[allow(non_camel_case_types)]
type c_size_t = u64;

// from C preprocessor macro

#[cold]
fn cold<T>(x: T) -> T {
    x
}

fn builtin_expect<T>(x: T, y: T) -> T
where
    T: PartialEq,
{
    if x == y {
        x
    } else {
        cold(x)
    }
}

#[inline]
pub(crate) fn js_unlikely(x: bool) -> bool {
    // use std::intrinsics::unlikely;
    // unsafe { unlikely(x) }
    builtin_expect(x, false)
}

#[allow(non_camel_case_types)]
pub type JS_BOOL = c_int;

#[inline]
#[allow(non_snake_case)]
pub const unsafe fn JS_VALUE_GET_TAG(v: JSValue) -> c_int {
    v.tag as c_int
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_VALUE_GET_PTR(v: JSValue) -> *mut c_void {
    v.u.ptr
}

#[inline]
#[allow(non_snake_case)]
pub const unsafe fn JS_MKVAL(tag: c_int, val: i32) -> JSValue {
    JSValue {
        u: JSValueUnion { int32: val as i32 },
        tag: tag as i64,
    }
}

#[inline]
#[allow(non_snake_case)]
pub const unsafe fn JS_MKPTR(tag: c_int, p: *mut c_void) -> JSValue {
    JSValue {
        u: JSValueUnion { ptr: p },
        tag: tag as i64,
    }
}

#[inline]
#[allow(non_snake_case)]
pub const unsafe fn JS_TAG_IS_FLOAT64(tag: i32) -> bool {
    tag == JS_TAG_FLOAT64
}

pub const JS_NAN: JSValue = JSValue {
    u: JSValueUnion { float64: f64::NAN },
    tag: JS_TAG_FLOAT64 as i64,
};

#[inline]
#[allow(non_snake_case)]
pub const unsafe fn JS_VALUE_HAS_REF_COUNT(v: JSValue) -> bool {
    JS_VALUE_GET_TAG(v) as u32 >= JS_TAG_FIRST as u32
}

pub const JS_NULL: JSValue = unsafe { JS_MKVAL(JS_TAG_NULL, 0) };
pub const JS_UNDEFINED: JSValue = unsafe { JS_MKVAL(JS_TAG_UNDEFINED, 0) };
pub const JS_FALSE: JSValue = unsafe { JS_MKVAL(JS_TAG_BOOL, 0) };
pub const JS_TRUE: JSValue = unsafe { JS_MKVAL(JS_TAG_BOOL, 1) };
pub const JS_EXCEPTION: JSValue = unsafe { JS_MKVAL(JS_TAG_EXCEPTION, 0) };
pub const JS_UNINITIALIZED: JSValue = unsafe { JS_MKVAL(JS_TAG_UNINITIALIZED, 0) };

// value handling

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewBool(_ctx: *mut JSContext, val: JS_BOOL) -> JSValue {
    JS_MKVAL(JS_TAG_BOOL, (val != 0) as i32)
}

macro_rules! def_js_new_some {
    ($name: ident, $tag: expr) => {
        #[inline]
        #[allow(non_snake_case)]
        pub unsafe fn $name(_ctx: *mut JSContext, val: i32) -> JSValue {
            JS_MKVAL($tag, val)
        }
    };
}

def_js_new_some!(JS_NewInt32, JS_TAG_INT);
def_js_new_some!(JS_NewCatchOffset, JS_TAG_CATCH_OFFSET);

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewInt64(ctx: *mut JSContext, val: i64) -> JSValue {
    let cast = val as i32;
    if val == cast as i64 {
        JS_NewInt32(ctx, cast)
    } else {
        JS_NewFloat64(ctx, val as f64)
    }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewUint32(ctx: *mut JSContext, val: u32) -> JSValue {
    if val <= 0x7fffffff {
        JS_NewInt32(ctx, val as i32)
    } else {
        JS_NewFloat64(ctx, val as f64)
    }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewFloat64(_ctx: *mut JSContext, val: f64) -> JSValue {
    JSValue {
        u: JSValueUnion { float64: val },
        tag: JS_TAG_FLOAT64 as i64,
    }
}

macro_rules! def_js_is_some {
    ($name: ident, $tag: expr) => {
        #[inline]
        #[allow(non_snake_case)]
        pub unsafe fn $name(v: JSValue) -> JS_BOOL {
            (JS_VALUE_GET_TAG(v) == $tag) as JS_BOOL
        }
    };
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsNumber(v: JSValue) -> JS_BOOL {
    let tag = JS_VALUE_GET_TAG(v);
    (tag == JS_TAG_INT || JS_TAG_IS_FLOAT64(tag)) as JS_BOOL
}

def_js_is_some!(JS_IsBigInt, JS_TAG_BIG_INT);
def_js_is_some!(JS_IsBigFloat, JS_TAG_BIG_FLOAT);
def_js_is_some!(JS_IsBigDecimal, JS_TAG_BIG_DECIMAL);
def_js_is_some!(JS_IsBool, JS_TAG_BOOL);
def_js_is_some!(JS_IsNull, JS_TAG_NULL);
def_js_is_some!(JS_IsUndefined, JS_TAG_UNDEFINED);

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_IsException(v: JSValue) -> JS_BOOL {
    js_unlikely(JS_VALUE_GET_TAG(v) == JS_TAG_EXCEPTION) as JS_BOOL
}

def_js_is_some!(JS_IsUninitialized, JS_TAG_UNINITIALIZED);
def_js_is_some!(JS_IsString, JS_TAG_STRING);
def_js_is_some!(JS_IsSymbol, JS_TAG_SYMBOL);
def_js_is_some!(JS_IsObject, JS_TAG_OBJECT);

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
pub unsafe fn JS_DupValueRT(_rt: *mut JSRuntime, v: JSValue) -> JSValue {
    if JS_VALUE_HAS_REF_COUNT(v) {
        let p = JS_VALUE_GET_PTR(v) as *mut JSRefCountHeader;
        let pref: &mut JSRefCountHeader = &mut *p;
        pref.ref_count += 1;
    }
    v
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ToUint32(ctx: *mut JSContext, pres: *mut u32, val: JSValue) -> c_int {
    JS_ToInt32(ctx, pres as *mut i32, val)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ToCStringLen(ctx: *mut JSContext, plen: *mut c_size_t, val1: JSValue) -> *const c_char {
    JS_ToCStringLen2(ctx, plen, val1, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ToCString(ctx: *mut JSContext, val1: JSValue) -> *const c_char {
    JS_ToCStringLen2(ctx, null_mut(), val1, 0)
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

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewCFunction(ctx: *mut JSContext, func: JSCFunction, name: *const c_char, length: c_int) -> JSValue {
    JS_NewCFunction2(ctx, func, name, length, JSCFunctionEnum_JS_CFUNC_generic, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_NewCFunctionMagic(
    ctx: *mut JSContext,
    func: JSCFunctionMagic,
    name: *const c_char,
    length: c_int,
    cproto: c_uint,
    magic: c_int,
) -> JSValue {
    let func: JSCFunction = transmute(func);
    JS_NewCFunction2(ctx, func, name, length, cproto, magic)
}

#[allow(non_snake_case)]
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
    JS_CFUNC_INTERNAL_DEF!(name, length, JSCFunctionEnum_JS_CFUNC_generic, generic, func1, 0)
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_MAGIC_DEF(
    name: *const c_char,
    length: u8,
    func1: JSCFunctionMagic,
    magic: i16,
) -> JSCFunctionListEntry {
    JS_CFUNC_INTERNAL_DEF!(
        name,
        length,
        JSCFunctionEnum_JS_CFUNC_generic_magic,
        generic_magic,
        func1,
        magic
    )
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_SPECIAL_DEF_constructor(
    name: *const c_char,
    length: u8,
    func1: JSCFunction,
) -> JSCFunctionListEntry {
    JS_CFUNC_INTERNAL_DEF!(
        name,
        length,
        JSCFunctionEnum_JS_CFUNC_constructor,
        constructor,
        func1,
        0
    )
}

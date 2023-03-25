#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

pub use libquickjs_sys::*;
use std::{
    mem::transmute,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr::null_mut,
};

use crate::internal::i32_as_c_int;

// basic type definitions

#[allow(non_camel_case_types)]
pub type c_size_t = size_t;

// from C preprocessor macro

#[cold]
#[inline]
fn cold<T>(x: T) -> T {
    x
}

#[inline]
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
        u: JSValueUnion {
            int32: i32_as_c_int(val),
        },
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
pub unsafe fn JS_SetProperty(ctx: *mut JSContext, this_obj: JSValue, prop: JSAtom, val: JSValue) -> c_int {
    JS_SetPropertyInternal(ctx, this_obj, prop, val, JS_PROP_THROW as i32)
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
    ($name: expr, $prop_flags: expr, $def_type: expr, $magic: expr, u: { func: { $length: expr, $cproto: expr, { $field: ident: $func1: expr } } }) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: $prop_flags as u8,
            def_type: $def_type as u8,
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

macro_rules! JS_CGETSET_INTERNAL_DEF {
    ($name: expr, $prop_flags: expr, $def_type: expr, $magic: expr, u: { getset: { get: { $get_field:ident: $get_value:expr }, set: { $set_field:ident: $set_value:expr } } }) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: $prop_flags as u8,
            def_type: $def_type as u8,
            magic: $magic as i16,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                getset: JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_2 {
                    get: JSCFunctionType { $get_field: $get_value },
                    set: JSCFunctionType { $set_field: $set_value },
                },
            },
        }
    };
}

macro_rules! JS_PROP_INTERNAL_DEF {
    ($name: expr, $prop_flags: expr, $def_type: expr, $magic: expr, u: { $prop_field:ident : $prop_value:expr }) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: $prop_flags as u8,
            def_type: $def_type as u8,
            magic: $magic as i16,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                $prop_field: $prop_value,
            },
        }
    };
}

macro_rules! JS_OBJECT_INTERNAL_DEF {
    ($name: expr, $prop_flags: expr, $def_type: expr, $magic: expr, u: { prop_list: { $tab: expr, $len: expr } }) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: $prop_flags as u8,
            def_type: $def_type as u8,
            magic: $magic as i16,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                prop_list: JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_4 { tab: $tab, len: $len },
            },
        }
    };
}

macro_rules! JS_ALIAS_INTERNAL_DEF {
    ($name: expr, $prop_flags: expr, $def_type: expr, $magic: expr, u: { alias: { $alias_name: expr, $base: expr } }) => {
        JSCFunctionListEntry {
            name: $name,
            prop_flags: $prop_flags as u8,
            def_type: $def_type as u8,
            magic: $magic as i16,
            u: JSCFunctionListEntry__bindgen_ty_1 {
                alias: JSCFunctionListEntry__bindgen_ty_1__bindgen_ty_3 {
                    name: $alias_name,
                    base: $base,
                },
            },
        }
    };
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_DEF(name: *const c_char, length: u8, func1: JSCFunction) -> JSCFunctionListEntry {
    // #define JS_CFUNC_DEF(name, length, func1) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, 0, .u = { .func = { length, JS_CFUNC_generic, { .generic = func1 } } } }
    JS_CFUNC_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, 0, u: { func: { length, JSCFunctionEnum_JS_CFUNC_generic, { generic: func1 } } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CFUNC_MAGIC_DEF(
    name: *const c_char,
    length: u8,
    func1: JSCFunctionMagic,
    magic: i16,
) -> JSCFunctionListEntry {
    // #define JS_CFUNC_MAGIC_DEF(name, length, func1, magic) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, magic, .u = { .func = { length, JS_CFUNC_generic_magic, { .generic_magic = func1 } } } }
    JS_CFUNC_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC,  magic, u: { func: { length, JSCFunctionEnum_JS_CFUNC_generic_magic, { generic_magic: func1 } } } }
}

#[allow(non_snake_case)]
macro_rules! def_JS_CFUNC_SPECIAL_DEF {
    ($name: ident, $cproto: ident, $field: ident, $func_type: ty) => {
        #[inline]
        #[allow(non_snake_case)]
        pub unsafe fn $name(
            name: *const c_char,
            length: u8,
            func1: $func_type,
        ) -> JSCFunctionListEntry {
            // #define JS_CFUNC_SPECIAL_DEF(name, length, cproto, func1) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, 0, .u = { .func = { length, JS_CFUNC_ ## cproto, { .cproto = func1 } } } }
            JS_CFUNC_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, 0, u: { func: { length, $cproto, { $field: func1 } } } }
        }
    }
}

def_JS_CFUNC_SPECIAL_DEF!(
    JS_CFUNC_CONSTRUCTOR_DEF,
    JSCFunctionEnum_JS_CFUNC_constructor,
    constructor,
    JSCFunction
);
def_JS_CFUNC_SPECIAL_DEF!(
    JS_CFUNC_CONSTRUCTOR_OR_FUNC_DEF,
    JSCFunctionEnum_JS_CFUNC_constructor_or_func,
    constructor_or_func,
    JSCFunction
);
def_JS_CFUNC_SPECIAL_DEF!(
    JS_CFUNC_F_F_DEF,
    JSCFunctionEnum_JS_CFUNC_f_f,
    f_f,
    Option<unsafe extern "C" fn(f64) -> f64>
);
def_JS_CFUNC_SPECIAL_DEF!(
    JS_CFUNC_F_F_F_DEF,
    JSCFunctionEnum_JS_CFUNC_f_f_f,
    f_f_f,
    Option<unsafe extern "C" fn(f64, f64) -> f64>
);

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ITERATOR_NEXT_DEF(
    name: *const c_char,
    length: u8,
    func1: Option<unsafe extern "C" fn(*mut JSContext, JSValue, c_int, *mut JSValue, *mut c_int, c_int) -> JSValue>,
    magic: i16,
) -> JSCFunctionListEntry {
    // #define JS_ITERATOR_NEXT_DEF(name, length, func1, magic) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC, magic, .u = { .func = { length, JS_CFUNC_iterator_next, { .iterator_next = func1 } } } }
    JS_CFUNC_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_CFUNC,  magic, u: { func: { length, JSCFunctionEnum_JS_CFUNC_iterator_next, { iterator_next: func1 } } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CGETSET_DEF(
    name: *const c_char,
    fgetter: Option<unsafe extern "C" fn(*mut JSContext, JSValue) -> JSValue>,
    fsetter: Option<unsafe extern "C" fn(*mut JSContext, JSValue, JSValue) -> JSValue>,
) -> JSCFunctionListEntry {
    // #define JS_CGETSET_DEF(name, fgetter, fsetter) { name, JS_PROP_CONFIGURABLE, JS_DEF_CGETSET, 0, .u = { .getset = { .get = { .getter = fgetter }, .set = { .setter = fsetter } } } }
    JS_CGETSET_INTERNAL_DEF! { name, JS_PROP_CONFIGURABLE, JS_DEF_CGETSET,  0, u: { getset: { get: { getter: fgetter }, set: { setter: fsetter } } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_CGETSET_MAGIC_DEF(
    name: *const c_char,
    fgetter: Option<unsafe extern "C" fn(*mut JSContext, JSValue, c_int) -> JSValue>,
    fsetter: Option<unsafe extern "C" fn(*mut JSContext, JSValue, JSValue, c_int) -> JSValue>,
    magic: i16,
) -> JSCFunctionListEntry {
    // #define JS_CGETSET_MAGIC_DEF(name, fgetter, fsetter, magic) { name, JS_PROP_CONFIGURABLE, JS_DEF_CGETSET_MAGIC, magic, .u = { .getset = { .get = { .getter_magic = fgetter }, .set = { .setter_magic = fsetter } } } }
    JS_CGETSET_INTERNAL_DEF! { name, JS_PROP_CONFIGURABLE, JS_DEF_CGETSET_MAGIC, magic, u: { getset: { get: { getter_magic: fgetter }, set: { setter_magic: fsetter } } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_PROP_STRING_DEF(name: *const c_char, cstr: *const c_char, prop_flags: u32) -> JSCFunctionListEntry {
    // #define JS_PROP_STRING_DEF(name, cstr, prop_flags) { name, prop_flags, JS_DEF_PROP_STRING, 0, .u = { .str = cstr } }
    JS_PROP_INTERNAL_DEF! { name, prop_flags, JS_DEF_PROP_STRING, 0, u: { str_: cstr } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_PROP_INT32_DEF(name: *const c_char, val: i32, prop_flags: u32) -> JSCFunctionListEntry {
    // #define JS_PROP_INT32_DEF(name, val, prop_flags) { name, prop_flags, JS_DEF_PROP_INT32, 0, .u = { .i32 = val } }
    JS_PROP_INTERNAL_DEF! { name, prop_flags, JS_DEF_PROP_INT32, 0, u: { i32_: val } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_PROP_INT64_DEF(name: *const c_char, val: i64, prop_flags: u32) -> JSCFunctionListEntry {
    // #define JS_PROP_INT64_DEF(name, val, prop_flags) { name, prop_flags, JS_DEF_PROP_INT64, 0, .u = { .i64 = val } }
    JS_PROP_INTERNAL_DEF! { name, prop_flags, JS_DEF_PROP_INT64, 0, u: { i64_: val } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_PROP_DOUBLE_DEF(name: *const c_char, val: f64, prop_flags: u32) -> JSCFunctionListEntry {
    // #define JS_PROP_DOUBLE_DEF(name, val, prop_flags) { name, prop_flags, JS_DEF_PROP_DOUBLE, 0, .u = { .f64 = val } }
    JS_PROP_INTERNAL_DEF! { name, prop_flags, JS_DEF_PROP_DOUBLE, 0, u: { f64_: val } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_PROP_UNDEFINED_DEF(name: *const c_char, prop_flags: u32) -> JSCFunctionListEntry {
    // #define JS_PROP_UNDEFINED_DEF(name, prop_flags) { name, prop_flags, JS_DEF_PROP_UNDEFINED, 0, .u = { .i32 = 0 } }
    JS_PROP_INTERNAL_DEF! { name, prop_flags, JS_DEF_PROP_UNDEFINED, 0, u: { i32_: 0 } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_OBJECT_DEF(
    name: *const c_char,
    tab: *const JSCFunctionListEntry,
    len: i32,
    prop_flags: u32,
) -> JSCFunctionListEntry {
    // #define JS_OBJECT_DEF(name, tab, len, prop_flags) { name, prop_flags, JS_DEF_OBJECT, 0, .u = { .prop_list = { tab, len } } }
    JS_OBJECT_INTERNAL_DEF! { name, prop_flags, JS_DEF_OBJECT, 0, u: { prop_list: { tab, len } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ALIAS_DEF(name: *const c_char, from: *const c_char) -> JSCFunctionListEntry {
    // #define JS_ALIAS_DEF(name, from) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_ALIAS, 0, .u = { .alias = { from, -1 } } }
    JS_ALIAS_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_ALIAS, 0, u: { alias: { from, -1 } } }
}

#[inline]
#[allow(non_snake_case)]
pub unsafe fn JS_ALIAS_BASE_DEF(name: *const c_char, from: *const c_char, base: i32) -> JSCFunctionListEntry {
    // #define JS_ALIAS_BASE_DEF(name, from, base) { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_ALIAS, 0, .u = { .alias = { from, base } } }
    JS_ALIAS_INTERNAL_DEF! { name, JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE, JS_DEF_ALIAS, 0, u: { alias: { from, base } } }
}

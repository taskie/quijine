use crate::{conversion::AsJSValue, ffi, Context, Value};
use std::{ffi::c_void, os::raw::c_int};

macro_rules! def_unpack_closure {
    ($name: ident $(, $param: ident : $type: ident)*) => {
        pub fn $name<$($type ,)* R, F>(closure: &mut F) -> (
            *mut std::ffi::c_void,
            unsafe extern "C" fn($($type, )* *mut std::ffi::c_void) -> R,
        )
        where
            F: FnMut($($type),*) -> R,
        {
            extern "C" fn trampoline<$($type ,)* R, F>($($param: $type, )* data: *mut std::ffi::c_void) -> R
            where
                F: FnMut($($type),*) -> R,
            {
                let closure: &mut F = unsafe { &mut *(data as *mut F) };
                (*closure)($($param),*)
            }

            (closure as *mut F as *mut std::ffi::c_void, trampoline::<$($type ,)* R, F>)
        }
    };
}

def_unpack_closure!(unpack_closure0);
def_unpack_closure!(unpack_closure1, t1: T1);
def_unpack_closure!(unpack_closure2, t1: T1, t2: T2);
def_unpack_closure!(unpack_closure3, t1: T1, t2: T2, t3: T3);
def_unpack_closure!(unpack_closure4, t1: T1, t2: T2, t3: T3, t4: T4);
def_unpack_closure!(unpack_closure5, t1: T1, t2: T2, t3: T3, t4: T4, t5: T5);

#[test]
fn test_unpack_closure() {
    let scale = 2;
    let (p, c) = unpack_closure0(Box::new(|| -1 * scale).as_mut());
    assert_eq!(-2, unsafe { c(p) });
    let (p, c) = unpack_closure1(Box::new(|x: i32| x * scale).as_mut());
    assert_eq!(2, unsafe { c(1, p) });
    let (p, c) = unpack_closure2(Box::new(|x: i32, y: i32| (x + y) * scale).as_mut());
    assert_eq!(6, unsafe { c(1, 2, p) });
    let (p, c) = unpack_closure3(Box::new(|x: i32, y: i32, z: i32| (x + y + z) * scale).as_mut());
    assert_eq!(12, unsafe { c(1, 2, 3, p) });
    let (p, c) = unpack_closure4(Box::new(|x: i32, y: i32, z: i32, w: i32| (x + y + z + w) * scale).as_mut());
    assert_eq!(20, unsafe { c(1, 2, 3, 4, p) });
    let (p, c) =
        unpack_closure5(Box::new(|x: i32, y: i32, z: i32, w: i32, v: i32| (x + y + z + w + v) * scale).as_mut());
    assert_eq!(30, unsafe { c(1, 2, 3, 4, 5, p) });
}

pub fn unpack_closure_to_c_function_data<F>(closure: &mut F) -> (ffi::JSCFunctionData, ffi::JSValue)
where
    F: FnMut(*mut ffi::JSContext, ffi::JSValue, c_int, *mut ffi::JSValue) -> ffi::JSValue,
{
    unsafe extern "C" fn trampoline<F>(
        ctx: *mut ffi::JSContext,
        this_val: ffi::JSValue,
        argc: c_int,
        argv: *mut ffi::JSValue,
        _magic: c_int,
        data: *mut ffi::JSValue,
    ) -> ffi::JSValue
    where
        F: FnMut(*mut ffi::JSContext, ffi::JSValue, c_int, *mut ffi::JSValue) -> ffi::JSValue,
    {
        let closure: &mut F = unsafe { &mut *((*data).u.ptr as *mut F) };
        (*closure)(ctx, this_val, argc, argv)
    }
    let value = unsafe { ffi::JS_MKPTR(ffi::JS_TAG_NULL, closure as *mut F as *mut std::ffi::c_void) };
    (Some(trampoline::<F>), value)
}

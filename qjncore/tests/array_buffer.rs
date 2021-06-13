use qjncore::{raw, Context, Runtime};
use std::{ffi::c_void, mem::size_of, ptr::null_mut};

#[derive(Debug, PartialEq, Eq)]
struct S1 {
    x: i32,
    y: i32,
    z: i32,
}

#[test]
fn test() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    let s1_box = Box::new(S1 { x: 42, y: 127, z: 0 });
    let s1_ptr = Box::into_raw(s1_box);
    unsafe extern "C" fn free_func(_rrt: *mut raw::JSRuntime, _opaque: *mut c_void, ptr: *mut c_void) {
        let s1_box = Box::from_raw(ptr as *mut S1);
        eprintln!("{:?}", s1_box);
        eprintln!("dropped: {:p}", ptr);
    }
    let ab = unsafe { ctx.new_array_buffer(s1_ptr as *mut u8, size_of::<S1>(), Some(free_func), null_mut(), false) };
    eprintln!("saved: {:p}", s1_ptr);
    let s1_buf_bytes = ab.array_buffer(ctx);
    assert_eq!(size_of::<S1>(), s1_buf_bytes.unwrap().len());
    let s1_ref: Option<&S1> = unsafe { ab.array_buffer_as_ref(ctx) };
    assert_eq!(Some(&S1 { x: 42, y: 127, z: 0 }), s1_ref);
    unsafe {
        ctx.free_value(ab);
        Context::free(ctx);
        Runtime::free(rt);
    }
}

#[test]
fn test_boxed() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    let s1_box = Box::new(S1 { x: 42, y: 127, z: 0 });
    let ab = ctx.new_array_buffer_from_boxed(s1_box);
    let s1_buf_bytes = ab.array_buffer(ctx);
    assert_eq!(size_of::<S1>(), s1_buf_bytes.unwrap().len());
    let s1_ref: Option<&S1> = unsafe { ab.array_buffer_as_ref(ctx) };
    assert_eq!(Some(&S1 { x: 42, y: 127, z: 0 }), s1_ref);
    unsafe {
        ctx.free_value(ab);
        Context::free(ctx);
        Runtime::free(rt);
    }
}

use crate::ffi::{self, c_size_t};
use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    ffi::c_void,
    mem::size_of_val,
    ptr::null_mut,
};

#[cfg(target_os = "macos")]
const MALLOC_OVERHEAD: c_size_t = 0;
#[cfg(not(target_os = "macos"))]
const MALLOC_OVERHEAD: c_size_t = 8;

unsafe extern "C" fn js_malloc(s: *mut ffi::JSMallocState, size: c_size_t) -> *mut c_void {
    let s = &mut *s;
    if s.malloc_size + size > s.malloc_limit {
        return null_mut();
    }
    let ptr = alloc(Layout::array::<u8>(size as usize).unwrap());
    if ptr.is_null() {
        return null_mut();
    }
    s.malloc_count += 1;
    s.malloc_size += js_malloc_usable_size(ptr.cast()) + MALLOC_OVERHEAD;
    ptr.cast()
}

unsafe extern "C" fn js_free(s: *mut ffi::JSMallocState, ptr: *mut c_void) {
    let s = &mut *s;
    if ptr.is_null() {
        return;
    }
    let layout = Layout::for_value(&*(ptr.cast::<u8>()));
    s.malloc_count -= 1;
    s.malloc_size -= layout.size() as c_size_t + MALLOC_OVERHEAD;
    dealloc(ptr.cast(), layout);
}

unsafe extern "C" fn js_realloc(s: *mut ffi::JSMallocState, ptr: *mut c_void, size: c_size_t) -> *mut c_void {
    let s = &mut *s;
    if ptr.is_null() {
        if size == 0 {
            return null_mut();
        }
        return js_malloc(s, size);
    }
    let layout = Layout::for_value(&*(ptr.cast::<u8>()));
    let old_size = layout.size() as c_size_t;
    if size == 0 {
        s.malloc_count -= 1;
        s.malloc_size -= old_size + MALLOC_OVERHEAD;
        dealloc(ptr.cast(), layout);
        return null_mut();
    }
    if s.malloc_size + size - old_size > s.malloc_limit {
        return null_mut();
    }
    let ptr = realloc(ptr.cast(), layout, size as usize);
    if ptr.is_null() {
        return null_mut();
    }
    s.malloc_size += js_malloc_usable_size(ptr.cast()) - old_size;
    ptr.cast()
}

unsafe extern "C" fn js_malloc_usable_size(ptr: *const c_void) -> c_size_t {
    if ptr.is_null() {
        return 0;
    }
    size_of_val(&*(ptr.cast::<u8>())) as c_size_t
}

pub(crate) const GLOBAL_ALLOCATOR_MALLOC_FUNCTIONS: ffi::JSMallocFunctions = ffi::JSMallocFunctions {
    js_malloc: Some(js_malloc),
    js_free: Some(js_free),
    js_realloc: Some(js_realloc),
    js_malloc_usable_size: Some(js_malloc_usable_size),
};

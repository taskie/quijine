use std::{mem, ptr};

pub(crate) fn to_vec<T: Sized>(v: T) -> Vec<u8> {
    let len = mem::size_of::<T>();
    let mut buf: Vec<u8> = vec![0; len];
    unsafe {
        ptr::write(buf.as_mut_ptr() as *mut T, v);
    };
    buf
}

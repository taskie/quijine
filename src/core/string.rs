use std::{ffi::CStr, marker::PhantomData, os::raw::c_char};

use crate::core::marker::Invariant;

#[derive(Clone, Copy, Debug)]
pub struct CString<'q>(*const c_char, Invariant<'q>);

impl<'q> CString<'q> {
    pub(crate) fn new(c_str: *const c_char) -> CString<'q> {
        CString(c_str, PhantomData)
    }

    pub fn raw(this: Self) -> *const c_char {
        this.0
    }

    pub fn to_str(self) -> Option<&'q str> {
        std::str::from_utf8(self.as_bytes()).ok()
    }

    pub fn as_bytes(self) -> &'q [u8] {
        let nulled = self.as_bytes_with_nul();
        &nulled[..nulled.len() - 1]
    }

    pub fn as_bytes_with_nul(self) -> &'q [u8] {
        unsafe { CStr::from_ptr(self.0).to_bytes_with_nul() }
    }
}

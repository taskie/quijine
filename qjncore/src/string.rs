use crate::{convert::AsJsCString, marker::Invariant};
use std::{ffi::CStr, marker::PhantomData, os::raw::c_char};

#[derive(Clone, Copy, Debug)]
pub struct CString<'q>(*const c_char, Invariant<'q>);

impl<'q> CString<'q> {
    pub(crate) fn new(c_str: *const c_char) -> CString<'q> {
        CString(c_str, PhantomData)
    }

    pub fn to_str(self) -> Option<&'q str> {
        std::str::from_utf8(self.to_bytes()).ok()
    }

    pub fn to_bytes(self) -> &'q [u8] {
        let nulled = self.to_bytes_with_nul();
        &nulled[..nulled.len() - 1]
    }

    pub fn to_bytes_with_nul(self) -> &'q [u8] {
        unsafe { CStr::from_ptr(self.0).to_bytes_with_nul() }
    }
}

impl<'q> AsJsCString<'q> for CString<'q> {
    #[inline]
    fn as_js_c_string(&self) -> *const c_char {
        self.0
    }
}

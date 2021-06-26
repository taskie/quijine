use crate::{convert::AsPtr, marker::Invariant};
use std::{ffi::CStr, marker::PhantomData, os::raw::c_char, ptr::NonNull};

#[derive(Clone, Copy, Debug)]
pub struct CString<'q>(NonNull<c_char>, Invariant<'q>);

/// Represents the return value of `JS_ToCString`.
impl<'q> CString<'q> {
    #[inline]
    pub(crate) fn new(c_str: *const c_char) -> Option<CString<'q>> {
        if !c_str.is_null() {
            Some(CString(
                unsafe { NonNull::new_unchecked(c_str as *mut c_char) },
                PhantomData,
            ))
        } else {
            None
        }
    }

    /// # Safety
    /// The underlying pointer must not be freed.
    #[inline]
    pub unsafe fn to_c_str(self) -> &'q CStr {
        CStr::from_ptr(self.0.as_ptr())
    }

    /// # Safety
    /// The underlying pointer must not be freed.
    #[inline]
    pub unsafe fn to_str(self) -> Option<&'q str> {
        self.to_c_str().to_str().ok()
    }
}

impl<'q> AsPtr<c_char> for CString<'q> {
    #[inline]
    fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }
}

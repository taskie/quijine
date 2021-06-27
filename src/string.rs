use crate::{error::Error, result::Result};
use quijine_core as qc;

#[derive(Debug)]
pub struct CString<'q> {
    value: qc::CString<'q>,
    context: qc::Context<'q>,
}

impl<'q> CString<'q> {
    #[inline]
    pub fn from_raw_parts(value: qc::CString<'q>, context: qc::Context<'q>) -> CString<'q> {
        CString { value, context }
    }

    unsafe fn free(this: &mut Self) {
        #[cfg(feature = "debug_leak")]
        log::trace!("free: {:?}", this.to_str());
        this.context.free_c_string(this.value)
    }

    #[inline]
    pub fn to_str(&self) -> Result<&str> {
        unsafe { self.value.to_str() }.ok_or_else(|| Error::with_str(crate::ErrorKind::InternalError, "invalid string"))
    }

    #[inline]
    pub fn to_string(&self) -> Result<String> {
        self.to_str().map(|s| s.to_owned())
    }
}

impl Drop for CString<'_> {
    fn drop(&mut self) {
        unsafe { CString::free(self) };
    }
}

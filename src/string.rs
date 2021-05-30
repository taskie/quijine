use crate::error::{Error, Result};
use qjncore as qc;

#[derive(Debug)]
pub struct CString<'q> {
    value: qc::CString<'q>,
    context: qc::Context<'q>,
}

impl<'q> CString<'q> {
    #[inline]
    pub fn from(value: qc::CString<'q>, context: qc::Context<'q>) -> CString<'q> {
        CString { value, context }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.value.as_bytes()
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    #[inline]
    pub fn to_str(&self) -> Result<&str> {
        self.value
            .to_str()
            .ok_or_else(|| Error::with_str(crate::ErrorKind::InternalError, "invalid string"))
    }

    #[inline]
    pub fn to_string(&self) -> Result<String> {
        self.to_str().map(|s| s.to_owned())
    }
}

impl Drop for CString<'_> {
    fn drop(&mut self) {
        log::debug!("drop: {:?}", self.to_str());
        unsafe { self.context.free_c_string(self.value) }
    }
}

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
    pub fn to_str(&self) -> Option<&str> {
        self.value.to_str()
    }

    #[inline]
    pub fn to_string(&self) -> Option<String> {
        self.to_str().map(|s| s.to_string())
    }
}

impl Drop for CString<'_> {
    fn drop(&mut self) {
        log::debug!("drop: {:?}", self.to_str());
        unsafe { self.context.free_c_string(self.value) }
    }
}

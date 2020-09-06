use quilt::{CString, Context};

#[derive(Debug)]
pub struct QjCString<'q> {
    value: CString<'q>,
    context: Context<'q>,
}

impl<'q> QjCString<'q> {
    #[inline]
    pub fn from(value: CString<'q>, context: Context<'q>) -> QjCString<'q> {
        QjCString { value, context }
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

impl Drop for QjCString<'_> {
    fn drop(&mut self) {
        log::debug!("drop: {:?}", self.to_str());
        unsafe { self.context.free_c_string(self.value) }
    }
}

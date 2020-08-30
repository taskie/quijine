use crate::{
    context::{QjContext, QjContextGuard},
    core::Runtime,
};
use std::{ffi::c_void, fmt, ptr::null_mut};

pub struct QjRuntimeOpaque {
    extra: *mut c_void,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QjRuntime<'r>(Runtime<'r>);

impl<'r> QjRuntime<'r> {
    #[inline]
    pub(crate) fn from(rt: Runtime<'r>) -> Self {
        QjRuntime(rt)
    }

    #[inline]
    pub(crate) fn into(self) -> Runtime<'r> {
        self.0
    }

    #[inline]
    pub fn new_context_guard(self) -> QjContextGuard<'r> {
        QjContextGuard::new(self)
    }

    #[inline]
    pub fn run_gc(self) {
        self.0.run_gc();
    }

    #[inline]
    pub(crate) fn opaque(&self) -> &QjRuntimeOpaque {
        unsafe { &*(self.0.opaque() as *mut QjRuntimeOpaque) }
    }

    #[inline]
    pub(crate) fn opaque_mut(&mut self) -> &mut QjRuntimeOpaque {
        unsafe { &mut *(self.0.opaque() as *mut QjRuntimeOpaque) }
    }
}

pub struct QjRuntimeGuard(QjRuntime<'static>);

impl QjRuntimeGuard {
    #[inline]
    pub fn new() -> Self {
        let rt = Runtime::new();
        let opaque = Box::new(QjRuntimeOpaque { extra: null_mut() });
        rt.set_opaque(Box::into_raw(opaque) as *mut c_void);
        QjRuntimeGuard(QjRuntime::from(rt))
    }

    #[inline]
    pub fn get(&self) -> QjRuntime<'static> {
        self.0
    }

    #[inline]
    pub fn new_context_guard(&self) -> QjContextGuard {
        QjContextGuard::new_with_guard(self)
    }

    #[inline]
    pub fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(QjRuntime) -> R,
    {
        f(self.0)
    }

    #[inline]
    pub fn run_with_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce(QjContext) -> R,
    {
        let ctx = self.new_context_guard();
        ctx.with(f)
    }
}

unsafe impl Send for QjRuntimeGuard {}

impl Drop for QjRuntimeGuard {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw((self.0).0.opaque() as *mut QjRuntimeOpaque);
            Runtime::free(self.0.into())
        }
    }
}

impl Default for QjRuntimeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for QjRuntimeGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("QjRuntimeGuard({:?})", self.0).as_str())
    }
}

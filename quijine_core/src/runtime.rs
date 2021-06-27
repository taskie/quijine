use crate::{
    atom::Atom,
    class::{ClassDef, ClassId},
    convert::{AsJsAtom, AsJsClassId, AsJsValue, AsMutPtr, AsPtr},
    ffi::{self, c_size_t},
    marker::Covariant,
    raw,
    value::Value,
    Context,
};
use std::{
    ffi::{c_void, CStr},
    fmt,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Runtime<'q>(NonNull<ffi::JSRuntime>, Covariant<'q>);

impl<'q> Runtime<'q> {
    // lifecycle

    /// # Safety
    /// The pointer of a runtime must have valid lifetime.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut ffi::JSRuntime) -> Runtime<'q> {
        Runtime(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Runtime<'q> {
        unsafe { Self::from_raw(ffi::JS_NewRuntime()) }
    }

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn new_2(mf: &'static ffi::JSMallocFunctions, opaque: *mut c_void) -> Runtime<'q> {
        unsafe { Self::from_raw(ffi::JS_NewRuntime2(mf, opaque)) }
    }

    /// # Safety
    /// You must free a runtime only once.
    #[inline]
    pub unsafe fn free(this: Self) {
        ffi::JS_FreeRuntime(this.0.as_ptr());
    }

    // basic

    #[inline]
    pub fn set_runtime_info(self, info: &'q CStr) {
        unsafe { ffi::JS_SetRuntimeInfo(self.0.as_ptr(), info.as_ptr()) }
    }

    #[inline]
    pub fn set_memory_limit(self, memory_limit: usize) {
        unsafe { ffi::JS_SetMemoryLimit(self.0.as_ptr(), memory_limit as c_size_t) }
    }

    #[inline]
    pub fn set_gc_threshold(self, gc_threshold: usize) {
        unsafe { ffi::JS_SetGCThreshold(self.0.as_ptr(), gc_threshold as c_size_t) }
    }

    #[inline]
    pub fn set_max_stack_size(self, stack_size: usize) {
        unsafe { ffi::JS_SetMaxStackSize(self.0.as_ptr(), stack_size as c_size_t) }
    }

    #[inline]
    pub fn opaque(self) -> *mut c_void {
        unsafe { ffi::JS_GetRuntimeOpaque(self.0.as_ptr()) }
    }

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetRuntimeOpaque(self.0.as_ptr(), opaque) }
    }

    #[inline]
    pub fn run_gc(self) {
        unsafe { ffi::JS_RunGC(self.0.as_ptr()) }
    }

    #[inline]
    pub fn is_live_object(self, value: Value) -> bool {
        unsafe { ffi::JS_IsLiveObject(self.0.as_ptr(), value.as_js_value()) != 0 }
    }

    // memory usage

    pub fn compute_memory_usage(self) -> raw::JSMemoryUsage {
        let mut s = MaybeUninit::<raw::JSMemoryUsage>::uninit();
        unsafe {
            ffi::JS_ComputeMemoryUsage(self.0.as_ptr(), s.as_mut_ptr());
            s.assume_init()
        }
    }

    // atom

    #[inline]
    pub fn free_atom(self, atom: Atom<'q>) {
        unsafe { ffi::JS_FreeAtomRT(self.0.as_ptr(), atom.as_js_atom()) };
    }

    // class

    #[inline]
    pub fn new_class(self, id: ClassId, class_def: &ClassDef) {
        unsafe {
            let result = ffi::JS_NewClass(self.0.as_ptr(), id.as_js_class_id(), &class_def.c_def());
            assert_eq!(0, result)
        }
    }

    #[inline]
    pub fn id_registered_class(self, id: ClassId) -> bool {
        unsafe { ffi::JS_IsRegisteredClass(self.0.as_ptr(), id.as_js_class_id()) != 0 }
    }

    // value

    /// # Safety
    /// You must free a value only once.
    #[inline]
    pub unsafe fn free_value(self, value: Value<'q>) {
        ffi::JS_FreeValueRT(self.0.as_ptr(), value.as_js_value());
    }

    #[inline]
    pub fn dup_value(self, value: Value<'q>) -> Value<'q> {
        unsafe {
            let res = ffi::JS_DupValueRT(self.0.as_ptr(), value.as_js_value());
            Value::from_raw_with_runtime(res, self)
        }
    }

    // ArrayBuffer

    #[inline]
    pub fn set_shared_array_buffer_functions(self, sf: &raw::JSSharedArrayBufferFunctions) {
        unsafe { ffi::JS_SetSharedArrayBufferFunctions(self.0.as_ptr(), sf) }
    }

    #[inline]
    pub fn set_can_block(self, can_block: bool) {
        unsafe { ffi::JS_SetCanBlock(self.0.as_ptr(), can_block.into()) }
    }

    // Promise

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_host_promise_rejection_tracker(self, cb: raw::JSHostPromiseRejectionTracker, opaque: *mut c_void) {
        unsafe { ffi::JS_SetHostPromiseRejectionTracker(self.0.as_ptr(), cb, opaque) }
    }

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_interrupt_handler(self, cb: raw::JSInterruptHandler, opaque: *mut c_void) {
        unsafe { ffi::JS_SetInterruptHandler(self.0.as_ptr(), cb, opaque) }
    }

    #[inline]
    pub fn is_job_pending(self) -> bool {
        unsafe { ffi::JS_IsJobPending(self.0.as_ptr()) != 0 }
    }

    /// return < 0 if exception, 0 if no job pending, 1 if a job was
    /// executed successfully.
    #[inline]
    pub fn execute_pending_job(self) -> (i32, Option<Context<'q>>) {
        let mut pctx: *mut ffi::JSContext = null_mut();
        let ret = unsafe { ffi::JS_ExecutePendingJob(self.0.as_ptr(), &mut pctx) as i32 };
        let ctx = if pctx.is_null() {
            None
        } else {
            Some(unsafe { Context::from_raw(pctx) })
        };
        (ret, ctx)
    }

    // Module

    /// module_normalize = NULL is allowed and invokes the default module
    /// filename normalizer
    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_module_loader_func(
        self,
        module_normalize: raw::JSModuleNormalizeFunc,
        module_loader: raw::JSModuleLoaderFunc,
        opaque: *mut c_void,
    ) {
        unsafe { ffi::JS_SetModuleLoaderFunc(self.0.as_ptr(), module_normalize, module_loader, opaque) }
    }
}

impl fmt::Debug for Runtime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Runtime({:p})", self.0).as_str())
    }
}

impl<'q> AsPtr<ffi::JSRuntime> for Runtime<'q> {
    #[inline]
    fn as_ptr(&self) -> *const ffi::JSRuntime {
        self.0.as_ptr()
    }
}

impl<'q> AsMutPtr<ffi::JSRuntime> for Runtime<'q> {
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut ffi::JSRuntime {
        self.0.as_ptr()
    }
}

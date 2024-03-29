use crate::{
    class::Class,
    context::{Context, ContextScope},
    result::Result,
};
use quijine_core as qc;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    ffi::{c_void, CString},
    fmt,
    result::Result as StdResult,
};

pub struct RuntimeOpaque {
    registered_classes: HashMap<TypeId, qc::ClassId>,
    class_defs: HashMap<qc::ClassId, qc::ClassDef>,
    class_names: HashSet<CString>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Runtime<'r>(qc::Runtime<'r>);

impl<'r> Runtime<'r> {
    #[inline]
    pub(crate) fn from(rt: qc::Runtime<'r>) -> Self {
        Runtime(rt)
    }

    #[inline]
    pub(crate) fn into(self) -> qc::Runtime<'r> {
        self.0
    }

    #[inline]
    pub fn new_context_scope(self) -> ContextScope<'r> {
        ContextScope::new(self)
    }

    #[inline]
    pub fn set_memory_limit(self, memory_limit: usize) {
        self.0.set_memory_limit(memory_limit);
    }

    #[inline]
    pub fn set_gc_threshold(self, gc_threshold: usize) {
        self.0.set_gc_threshold(gc_threshold);
    }

    #[inline]
    pub fn set_max_stack_size(self, stack_size: usize) {
        self.0.set_max_stack_size(stack_size)
    }

    #[inline]
    pub fn run_gc(self) {
        self.0.run_gc();
    }

    #[inline]
    pub(crate) fn opaque(&self) -> &RuntimeOpaque {
        unsafe { &*(self.0.opaque() as *mut RuntimeOpaque) }
    }

    #[inline]
    pub(crate) fn opaque_mut(&mut self) -> &mut RuntimeOpaque {
        unsafe { &mut *(self.0.opaque() as *mut RuntimeOpaque) }
    }

    #[inline]
    pub(crate) fn new_class(&self, id: qc::ClassId, class_def: &qc::ClassDef) {
        self.0.new_class(id, class_def)
    }

    pub(crate) fn class_id<T: Class + 'static>(&self) -> Option<qc::ClassId> {
        self.opaque().registered_classes.get(&TypeId::of::<T>()).cloned()
    }

    pub(crate) fn get_or_register_class_id<T: Class + 'static>(&mut self) -> qc::ClassId {
        let class_id = self.class_id::<T>();
        if let Some(class_id) = class_id {
            return class_id;
        }
        let class_id = qc::ClassId::generate();
        self.opaque_mut().registered_classes.insert(TypeId::of::<T>(), class_id);
        class_id
    }

    pub(crate) fn register_class_name(&mut self, class_name: CString) {
        self.opaque_mut().class_names.insert(class_name);
    }

    pub(crate) fn class_name(&self, class_name: &CString) -> Option<&std::ffi::CString> {
        self.opaque().class_names.get(class_name)
    }

    pub(crate) fn register_class_def(&mut self, class_id: qc::ClassId, class_def: qc::ClassDef) {
        self.opaque_mut().class_defs.insert(class_id, class_def);
    }

    pub(crate) fn class_def(&self, class_id: qc::ClassId) -> Option<&qc::ClassDef> {
        self.opaque().class_defs.get(&class_id)
    }
}

pub struct RuntimeScope(Runtime<'static>);

impl RuntimeScope {
    #[inline]
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        let rt = qc::Runtime::new();
        #[cfg(not(target_os = "windows"))]
        let rt = qc::Runtime::new_2(&qc::alloc::GLOBAL_ALLOCATOR_MALLOC_FUNCTIONS, std::ptr::null_mut());
        let opaque = Box::new(RuntimeOpaque {
            registered_classes: HashMap::new(),
            class_defs: HashMap::new(),
            class_names: HashSet::new(),
        });
        rt.set_opaque(Box::into_raw(opaque) as *mut c_void);
        RuntimeScope(Runtime::from(rt))
    }

    #[inline]
    pub fn get(&self) -> Runtime<'static> {
        self.0
    }

    #[inline]
    pub fn new_context_scope(&self) -> ContextScope {
        ContextScope::new_with_scope(self)
    }

    #[inline]
    pub fn run<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(Runtime) -> Result<R>,
    {
        f(self.0)
    }

    #[inline]
    pub fn run_with_context<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(Context) -> Result<R>,
    {
        let ctx = self.new_context_scope();
        ctx.with(f)
    }
}

impl Drop for RuntimeScope {
    fn drop(&mut self) {
        unsafe {
            // opaque must be bound until values in the runtime will be freed
            let _opaque = Box::from_raw((self.0).0.opaque() as *mut RuntimeOpaque);
            qc::Runtime::free(self.0.into())
        }
    }
}

impl Default for RuntimeScope {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RuntimeScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(format!("RuntimeScope({:?})", self.0).as_str())
    }
}

use crate::{Atom, Context, Result, Value};
use quijine_core::{self as qc};

#[cfg(feature = "c_function_list")]
use crate::{function::c_function_list_as_raw, CFunctionListEntry};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleDef<'q>(qc::ModuleDef<'q>, qc::Context<'q>);

impl<'q> ModuleDef<'q> {
    /// # Safety
    /// module must have the same lifetime as a context.
    #[inline]
    pub unsafe fn from_raw_parts(m: qc::ModuleDef<'q>, ctx: qc::Context<'q>) -> ModuleDef<'q> {
        ModuleDef(m, ctx)
    }

    fn context(self) -> Context<'q> {
        Context::from_raw(self.1)
    }

    /// return the import.meta object of a module
    #[inline]
    pub fn import_meta(self) -> Result<Value<'q>> {
        unsafe { self.context().wrap_result(self.0.import_meta(self.1)) }
    }

    #[inline]
    pub fn module_name(self) -> Atom<'q> {
        Atom::from_raw_parts(self.0.module_name(self.1), self.1)
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn add_module_export(self, name_str: &str) -> i32 {
        self.0.add_module_export(self.1, name_str)
    }

    /// can only be called before the module is instantiated
    #[cfg(feature = "c_function_list")]
    #[inline]
    pub fn add_module_export_list(self, tab: &[CFunctionListEntry]) -> i32 {
        self.0.add_module_export_list(self.1, c_function_list_as_raw(tab))
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn set_module_export(self, export_name: &str, val: Value<'q>) -> i32 {
        self.0.set_module_export(self.1, export_name, *val.as_raw())
    }

    /// can only be called before the module is instantiated
    #[cfg(feature = "c_function_list")]
    #[inline]
    pub fn set_module_export_list(self, tab: &[CFunctionListEntry]) -> i32 {
        self.0.set_module_export_list(self.1, c_function_list_as_raw(tab))
    }
}

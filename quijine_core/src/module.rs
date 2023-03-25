use crate::{
    atom::Atom,
    context::Context,
    convert::{AsMutPtr, AsPtr},
    ffi,
    function::c_function_list_as_ptr,
    internal::c_int_as_i32,
    marker::Covariant,
    value::Value,
    AsJsValue, CFunctionListEntry,
};
use std::{ffi::CString, marker::PhantomData, ptr::NonNull};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ModuleDef<'q>(NonNull<ffi::JSModuleDef>, Covariant<'q>);

impl<'q> ModuleDef<'q> {
    /// # Safety
    /// module must have the same lifetime as a context.
    #[inline]
    pub unsafe fn from_raw(m: *mut ffi::JSModuleDef, _ctx: Context<'q>) -> ModuleDef<'q> {
        ModuleDef(NonNull::new(m).unwrap(), PhantomData)
    }

    /// return the import.meta object of a module
    #[inline]
    pub fn import_meta(self, mut ctx: Context<'q>) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_GetImportMeta(ctx.as_mut_ptr(), self.0.as_ptr()), ctx) }
    }

    #[inline]
    pub fn module_name(self, mut ctx: Context<'q>) -> Atom<'q> {
        unsafe { Atom::from_raw(ffi::JS_GetModuleName(ctx.as_mut_ptr(), self.0.as_ptr()), ctx) }
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn add_module_export(self, mut ctx: Context<'q>, name_str: &str) -> i32 {
        let name_str = CString::new(name_str).unwrap();
        unsafe {
            c_int_as_i32(ffi::JS_AddModuleExport(
                ctx.as_mut_ptr(),
                self.0.as_ptr(),
                name_str.as_ptr(),
            ))
        }
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn add_module_export_list(self, mut ctx: Context<'q>, tab: &[CFunctionListEntry]) -> i32 {
        unsafe {
            c_int_as_i32(ffi::JS_AddModuleExportList(
                ctx.as_mut_ptr(),
                self.0.as_ptr(),
                c_function_list_as_ptr(tab),
                tab.len() as i32,
            ))
        }
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn set_module_export(self, mut ctx: Context<'q>, export_name: &str, val: Value<'q>) -> i32 {
        let export_name = CString::new(export_name).unwrap();
        unsafe {
            c_int_as_i32(ffi::JS_SetModuleExport(
                ctx.as_mut_ptr(),
                self.0.as_ptr(),
                export_name.as_ptr(),
                val.as_js_value(),
            ))
        }
    }

    /// can only be called before the module is instantiated
    #[inline]
    pub fn set_module_export_list(self, mut ctx: Context<'q>, tab: &[CFunctionListEntry]) -> i32 {
        unsafe {
            c_int_as_i32(ffi::JS_SetModuleExportList(
                ctx.as_mut_ptr(),
                self.0.as_ptr(),
                c_function_list_as_ptr(tab),
                tab.len() as i32,
            ))
        }
    }
}

impl<'q> AsPtr<ffi::JSModuleDef> for ModuleDef<'q> {
    #[inline]
    fn as_ptr(&self) -> *const ffi::JSModuleDef {
        self.0.as_ptr()
    }
}

impl<'q> AsMutPtr<ffi::JSModuleDef> for ModuleDef<'q> {
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut ffi::JSModuleDef {
        self.0.as_ptr()
    }
}

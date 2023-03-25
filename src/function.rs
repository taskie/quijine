use crate::{arena::CStringArena, PropFlags};
use quijine_core::{self as qc, raw};
use std::{mem::transmute, os::raw::c_int};

#[derive(Clone)]
#[repr(transparent)]
pub struct CFunctionListEntry(qc::CFunctionListEntry);

impl CFunctionListEntry {
    /// # Safety
    /// The lifetime must be valid.
    pub unsafe fn from_raw(raw: qc::CFunctionListEntry) -> CFunctionListEntry {
        CFunctionListEntry(raw)
    }
}

pub struct CFunctionListBuilder<'a> {
    arena: &'a mut CStringArena,
    vec: Vec<CFunctionListEntry>,
}

macro_rules! impl_def {
    ($vis: vis fn $name: ident(mut self, name: &str, $($arg: ident : $ty: ty),*) -> Self) => {
        #[inline]
        #[allow(unused_unsafe)]
        pub fn $name(mut self, name: &str, $($arg: $ty),*) -> Self {
            let name = self.arena.registered(name.to_owned());
            self.vec.push(CFunctionListEntry(unsafe { qc::CFunctionListEntry::$name(name, $($arg),*) }));
            self
        }
    };
}

macro_rules! impl_defs {
    ($($vis: vis fn $name: ident(mut self, name: &str, $($arg: ident : $ty: ty),*) -> Self ;)*) => {
        $(impl_def!(fn $name(mut self, name: &str, $($arg: $ty),*) -> Self);)*
    };
}

impl<'a> CFunctionListBuilder<'a> {
    impl_defs! {
        pub fn cfunc_def(mut self, name: &str, length: u8, func1: raw::JSCFunction) -> Self;
        pub fn cfunc_magic_def(mut self, name: &str, length: u8, func1:raw::JSCFunctionMagic, magic: i16) -> Self;
        pub fn cfunc_constructor_def(mut self, name: &str, length: u8, func1: raw::JSCFunction) -> Self;
        pub fn cfunc_constructor_or_func_def(mut self, name: &str, length: u8, func1: raw::JSCFunction) -> Self;
        pub fn cfunc_f_f_def(mut self, name: &str, length: u8, func1: Option<unsafe extern "C" fn(f64) -> f64>) -> Self;
        pub fn cfunc_f_f_f_def(mut self, name: &str, length: u8, func1: Option<unsafe extern "C" fn(f64, f64) -> f64>) -> Self;
        pub fn iterator_next_def(
            mut self,
            name: &str,
            length: u8,
            func1: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, c_int, *mut raw::JSValue, *mut c_int, c_int) -> raw::JSValue>,
            magic: i16
        ) -> Self;
        pub fn cgetset_def(
            mut self,
            name: &str,
            fgetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue) -> raw::JSValue>,
            fsetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, raw::JSValue) -> raw::JSValue>
        ) -> Self;
        pub fn cgetset_magic_def(
            mut self,
            name: &str,
            fgetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, c_int) -> raw::JSValue>,
            fsetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, raw::JSValue, c_int) -> raw::JSValue>,
            magic: i16
        ) -> Self;
        pub fn prop_int32_def(mut self, name: &str, val: i32, prop_flags: PropFlags) -> Self;
        pub fn prop_int64_def(mut self, name: &str, val: i64, prop_flags: PropFlags) -> Self;
        pub fn prop_double_def(mut self, name: &str, val: f64, prop_flags: PropFlags) -> Self;
        pub fn prop_undefined_def(mut self, name: &str, prop_flags: PropFlags) -> Self;
    }

    pub fn prop_string_def(mut self, name: &str, val: &str, prop_flags: PropFlags) -> Self {
        let val = self.arena.registered(val.to_owned()).to_owned();
        let name = self.arena.registered(name.to_owned());
        self.vec
            .push(CFunctionListEntry(qc::CFunctionListEntry::prop_string_def(
                name, &val, prop_flags,
            )));
        self
    }

    pub fn object_def(mut self, name: &str, tab: &[CFunctionListEntry], prop_flags: PropFlags) -> Self {
        let name = self.arena.registered(name.to_owned());
        self.vec.push(CFunctionListEntry(qc::CFunctionListEntry::object_def(
            name,
            c_function_list_as_raw(tab),
            prop_flags,
        )));
        self
    }

    pub fn alias_def(mut self, name: &str, from: &str) -> Self {
        let from = self.arena.registered(from.to_owned()).to_owned();
        let name = self.arena.registered(name.to_owned());
        self.vec
            .push(CFunctionListEntry(qc::CFunctionListEntry::alias_def(name, &from)));
        self
    }

    pub fn alias_base_def(mut self, name: &str, from: &str, base: i32) -> Self {
        let from = self.arena.registered(from.to_owned()).to_owned();
        let name = self.arena.registered(name.to_owned());
        self.vec.push(CFunctionListEntry(qc::CFunctionListEntry::alias_base_def(
            name, &from, base,
        )));
        self
    }

    pub fn new(arena: &'a mut CStringArena) -> Self {
        CFunctionListBuilder { arena, vec: vec![] }
    }

    pub fn build(self) -> Vec<CFunctionListEntry> {
        self.vec
    }
}

pub(crate) fn c_function_list_as_raw(list: &[CFunctionListEntry]) -> &[qc::CFunctionListEntry] {
    // this operation is safe because of repr(transparent)
    unsafe { transmute(list) }
}

#[cfg(test)]
mod tests {
    use crate::{arena::CStringArena, js_c_function, CFunctionListBuilder};

    #[test]
    fn test() {
        let mut arena = CStringArena::new();
        let _vec = CFunctionListBuilder::new(&mut arena)
            .cfunc_def("name", 0, js_c_function!(|ctx, _this, _args| ctx.undefined().into()))
            .build();
    }
}

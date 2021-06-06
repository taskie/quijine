use std::marker::PhantomData;

use crate::{context::Context, convert::AsJsAtom, ffi, marker::Invariant};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Atom<'q>(ffi::JSAtom, Invariant<'q>);

impl<'q> Atom<'q> {
    /// # Safety
    /// An atom must have the same lifetime as a context.
    pub unsafe fn from_raw(v: ffi::JSAtom, _ctx: Context<'q>) -> Self {
        Atom(v, PhantomData)
    }

    pub fn is_null(self) -> bool {
        self.0 == ffi::JS_ATOM_NULL
    }
}

impl<'q> AsJsAtom<'q> for Atom<'q> {
    #[inline]
    fn as_js_atom(&self) -> ffi::JSAtom {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct PropertyEnum<'q>(ffi::JSPropertyEnum, Invariant<'q>);

impl<'q> PropertyEnum<'q> {
    /// # Safety
    /// An atom must have the same lifetime as a context.
    pub unsafe fn from_raw(v: ffi::JSPropertyEnum, _ctx: Context<'q>) -> Self {
        PropertyEnum(v, PhantomData)
    }

    pub fn is_enumerable(&self) -> bool {
        self.0.is_enumerable != 0
    }

    pub fn atom(&self) -> Atom<'q> {
        Atom(self.0.atom, self.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Runtime};

    #[test]
    fn test() {
        let rt = Runtime::new();
        let ctx = Context::new(rt);
        let global = ctx.global_object();
        let k_foo = ctx.new_atom("foo");
        assert!(!k_foo.is_null());
        let value = ctx.new_int32(42);
        ctx.dup_value(value);
        assert!(!global.has_property(ctx, k_foo).unwrap());
        global.set_property(ctx, k_foo, value).unwrap();
        assert!(global.has_property(ctx, k_foo).unwrap());
        let value2 = global.property(ctx, k_foo);
        assert_eq!(42, value2.to_i32(ctx).unwrap());
        unsafe {
            ctx.free_value(value2);
            ctx.free_value(value);
            ctx.free_atom(k_foo);
            ctx.free_value(global);
            Context::free(ctx);
            Runtime::free(rt);
        }
    }
}

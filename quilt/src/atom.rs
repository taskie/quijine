use crate::{conversion::AsJsAtom, ffi, marker::Invariant};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Atom<'q>(u32, Invariant<'q>);

impl<'q> Atom<'q> {
    pub fn raw(this: Self) -> u32 {
        this.0
    }
}

impl<'q> AsJsAtom<'q> for Atom<'q> {
    #[inline]
    fn as_js_atom(&self) -> ffi::JSAtom {
        self.0
    }
}

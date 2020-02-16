use bitflags::_core::{cell::Cell, marker::PhantomData};

pub(crate) type Invariant<'a> = PhantomData<Cell<&'a ()>>;
pub(crate) type Covariant<'a> = PhantomData<&'a ()>;

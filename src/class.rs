use crate::{instance::Data, types::Object, Context, Result};
use std::panic::UnwindSafe;

pub trait ClassMethods<'q, T: Class> {
    fn add_method<F, R>(&mut self, name: &str, method: F)
    where
        F: Fn(Context<'q>, &mut T, Data<'q>, &[Data<'q>]) -> Result<R> + UnwindSafe + Send + 'static,
        R: Into<Data<'q>> + 'q;
}

pub trait Class: Sized {
    fn name() -> &'static str;
    fn add_methods<'q, T: ClassMethods<'q, Self>>(_methods: &mut T) {}
    fn setup_proto(_ctx: Context, _proto: &Object) {}
}

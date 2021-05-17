use crate::{
    instance::Qj,
    tags::{QjAnyTag, QjObjectTag},
    QjContext, QjResult, QjVec,
};
use std::marker::Sync;

pub trait QjClassMethods<'q, T: QjClass> {
    fn add_method<F>(&mut self, name: &str, method: F)
    where
        F: 'static
            + Send
            + Fn(QjContext<'q>, &mut T, Qj<'q, QjAnyTag>, QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>>
            + Sync;
}

pub trait QjClass: Sized {
    fn name() -> &'static str;
    fn add_methods<'q, T: QjClassMethods<'q, Self>>(_methods: &mut T) {}
    fn setup_proto(_ctx: QjContext, _proto: &Qj<QjObjectTag>) {}
}

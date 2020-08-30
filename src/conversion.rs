use crate::{context::QjContext, error::QjResult, string::QjCString, tags::QjVariant, QjError};

pub trait ToQj<'q> {
    fn to_qj(self, ctx: QjContext<'q>) -> QjResult<QjVariant<'q>>;
}

pub trait FromQj<'q>: Sized {
    fn from_qj(var: QjVariant<'q>, ctx: QjContext<'q>) -> QjResult<'q, Self>;
}

impl<'q> ToQj<'q> for QjVariant<'q> {
    fn to_qj(self, _ctx: QjContext<'q>) -> QjResult<'q, QjVariant<'q>> {
        Ok(self)
    }
}

impl<'q> FromQj<'q> for QjVariant<'q> {
    fn from_qj(var: QjVariant<'q>, _ctx: QjContext<'q>) -> QjResult<'q, Self> {
        Ok(var)
    }
}

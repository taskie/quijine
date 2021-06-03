use crate::data::Data;

pub trait AsData<'q> {
    fn as_data(&self) -> &Data<'q>;
}

impl<'q> AsData<'q> for Data<'q> {
    fn as_data(&self) -> &Data<'q> {
        self
    }
}

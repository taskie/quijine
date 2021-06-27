use crate::error::{Error, ExternalError};
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

pub trait ExternalResult<T> {
    fn map_err_to_qj(self) -> Result<T>;
}

impl<T, E> ExternalResult<T> for StdResult<T, E>
where
    E: ExternalError,
{
    fn map_err_to_qj(self) -> Result<T> {
        self.map_err(|e| e.to_qj_err())
    }
}

use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    #[doc(hidden)]
    HasException,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HasException => f.write_str("HasException"),
        }
    }
}

impl std::error::Error for Error {}

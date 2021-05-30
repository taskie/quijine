use crate::Data;
use std::{error::Error as StdError, fmt, io, result::Result as StdResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    EvalError,
    RangeError,
    ReferenceError,
    SyntaxError,
    TypeError,
    URIError,
    InternalError,
    AggregateError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ErrorKind::EvalError => "EvalError",
            ErrorKind::RangeError => "RangeError",
            ErrorKind::ReferenceError => "ReferenceError",
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::URIError => "URIError",
            ErrorKind::InternalError => "InternalError",
            ErrorKind::AggregateError => "AggregateError",
        })
    }
}

#[derive(Debug)]
pub struct JsErrorData {
    pub name: Option<String>,
    pub message: Option<String>,
    pub file_name: Option<String>,
    pub line_number: Option<i32>,
    pub stack: Option<String>,
}

impl fmt::Display for JsErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        let empty = &"".to_owned();
        let name = self.name.as_ref().unwrap_or(empty);
        let message = self.message.as_ref().unwrap_or(empty);
        f.write_fmt(format_args!("{}: {}", name, message))?;
        if let Some(stack) = self.stack.as_ref() {
            f.write_fmt(format_args!("\n{}", stack))?;
        }
        Ok(())
    }
}

pub enum ErrorValue {
    None,
    String(String),
    JsError(JsErrorData),
    External(Box<dyn StdError + Send + Sync + 'static>),
}

impl fmt::Display for ErrorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        let default = "";
        match self {
            ErrorValue::None => f.write_str(default),
            ErrorValue::String(s) => f.write_str(s),
            ErrorValue::JsError(e) => e.fmt(f),
            ErrorValue::External(e) => e.fmt(f),
        }
    }
}

impl fmt::Debug for ErrorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

pub struct Error {
    pub kind: ErrorKind,
    pub value: ErrorValue,
}

impl Error {
    pub fn from(kind: ErrorKind) -> Error {
        Error {
            kind,
            value: ErrorValue::None,
        }
    }

    pub fn with_str<T: AsRef<str>>(kind: ErrorKind, message: T) -> Error {
        Error {
            kind,
            value: ErrorValue::String(message.as_ref().to_string()),
        }
    }

    pub fn with_external(kind: ErrorKind, external: Box<dyn StdError + Send + Sync + 'static>) -> Error {
        Error {
            kind,
            value: ErrorValue::External(external),
        }
    }

    pub fn from_data<'q>(kind: ErrorKind, data: impl Into<Data<'q>>) -> Error {
        let data: Data<'q> = data.into();
        let ctx = data.context();
        let json = ctx
            .json_stringify(data, ctx.undefined(), ctx.undefined())
            .and_then(|s| s.to_string());
        Error {
            kind,
            value: ErrorValue::String(match json {
                Ok(s) => s,
                Err(e) => format!(r##"{{"name":"SystemError","message":"can't convert error: {}"}}"##, e),
            }),
        }
    }

    pub fn from_js_error<'q>(kind: ErrorKind, data: impl Into<Data<'q>>) -> Error {
        let data: Data<'q> = data.into();
        let data = JsErrorData {
            name: data.get("name").to_string().ok(),
            message: data.get("message").to_string().ok(),
            file_name: data.get("fileName").to_string().ok(),
            line_number: data.get("lineNumber").to_i32().ok(),
            stack: data.get("stack").to_string().ok(),
        };
        Error {
            kind,
            value: ErrorValue::JsError(data),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

impl<'q> StdError for Error {}

impl std::convert::From<Error> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::with_external(ErrorKind::InternalError, Box::new(err))
    }
}

impl From<Box<dyn StdError>> for std::boxed::Box<Error> {
    fn from(e: Box<dyn StdError>) -> Self {
        Box::new(Error::with_str(ErrorKind::InternalError, format!("{}", e)))
    }
}

impl From<Box<dyn StdError + Send + Sync + 'static>> for std::boxed::Box<Error> {
    fn from(e: Box<dyn StdError + Send + Sync + 'static>) -> Self {
        Box::new(Error::with_external(ErrorKind::InternalError, e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_fmt(format_args!("{}Error({})", self.kind, self.value))
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

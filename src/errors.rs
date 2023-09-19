//! Custom Error/Result for sqlite-loadable-rs APIs.
use std::{
    ffi::NulError,
    fmt,
    result,
};

/// A type alias for `Result<T, xxx::Error>`.
pub type Result<T> = result::Result<T, Error>;

/// Any error that occurs while creating or using a SQLite extension.
#[derive(Debug, PartialEq, Eq)]
pub struct Error(Box<ErrorKind>);

/// Generic Error
impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }
    pub fn new_message<S: AsRef<str>>(message: S) -> Error {
        Error(Box::new(ErrorKind::Message(message.as_ref().to_owned())))
    }

    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }

    pub fn code(self) -> i32 {
        1
    }
    pub fn code_extended(self) -> i32 {
        1
    }
    pub fn result_error_message(self) -> String {
        match *self.0 {
            ErrorKind::DefineVfs(i) => format!("Error resulted after calling function: {}", i),
            ErrorKind::DefineScalarFunction(_) => "Error defining scalar function".to_owned(),
            ErrorKind::CStringError(e) => format!("String Nul error: {}", e),
            ErrorKind::CStringUtf8Error(_) => "utf8 err".to_owned(),
            ErrorKind::Message(msg) => msg,
            ErrorKind::TableFunction(_) => "table func error".to_owned(),
        }
    }
}

/// The specific type of an error.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    DefineVfs(i32),
    DefineScalarFunction(i32),
    CStringError(NulError),
    CStringUtf8Error(std::str::Utf8Error),
    TableFunction(i32),
    Message(String),
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Error {
        Error::new(ErrorKind::CStringError(err))
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::new(ErrorKind::CStringUtf8Error(err))
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Error {
        Error::new_message(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::new_message(err.as_str())
    }
}

impl ErrorKind {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::DefineScalarFunction(ref err) => err.fmt(f),
            ErrorKind::DefineVfs(i) => f.write_fmt(format_args!("Define vfs error: {}", i)), // TODO test
            _ => unreachable!(),
        }
    }
}

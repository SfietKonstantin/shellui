use std::error::Error as StdError;
use std::fmt;
use std::io::{Error, ErrorKind};
use thiserror::Error;

pub type ShellUiResult<T> = Result<T, ShellUiError>;

#[derive(Debug, Error)]
pub enum ShellUiError {
    #[error(transparent)]
    Error(Error),
    #[error("{}", .0)]
    Warning(String),
    #[error("Interrupt")]
    Interrupt,
}

impl From<Error> for ShellUiError {
    fn from(error: Error) -> Self {
        match error.kind() {
            ErrorKind::Interrupted => ShellUiError::Interrupt,
            _ => ShellUiError::Error(error),
        }
    }
}

impl ShellUiError {
    pub fn warning<S>(message: S) -> Self
    where
        S: ToString,
    {
        ShellUiError::Warning(message.to_string())
    }

    pub fn interrupt() -> Self {
        ShellUiError::Interrupt
    }
}

pub trait WithContext {
    type Output;
    fn with_context<S>(self, context: S) -> Self::Output
    where
        S: ToString;
}

impl<T> WithContext for Option<T> {
    type Output = Result<T, Error>;
    fn with_context<S>(self, context: S) -> Self::Output
    where
        S: ToString,
    {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::other(context.to_string())),
        }
    }
}

impl<T, E> WithContext for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    type Output = Result<T, Error>;
    fn with_context<S>(self, context: S) -> Self::Output
    where
        S: ToString,
    {
        self.map_err(|error| error.with_context(context))
    }
}

pub trait WithContextError {
    fn with_context<S>(self, context: S) -> Error
    where
        S: ToString;
}

impl<E> WithContextError for E
where
    E: StdError + Send + Sync + 'static,
{
    fn with_context<S>(self, context: S) -> Error
    where
        S: ToString,
    {
        Error::other(ErrorWrapper::new(context.to_string(), self))
    }
}

#[derive(Debug)]
struct ErrorWrapper<E> {
    message: String,
    source: E,
}

impl<E> ErrorWrapper<E> {
    fn new(message: String, source: E) -> Self {
        ErrorWrapper { message, source }
    }
}

impl<E> fmt::Display for ErrorWrapper<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<E> StdError for ErrorWrapper<E>
where
    E: StdError + Send + Sync + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

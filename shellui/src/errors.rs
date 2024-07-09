use std::error::Error as StdError;
use std::fmt;
use std::io::Error;

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
        self.map_err(|error| with_context(error, context))
    }
}

fn with_context<E, S>(error: E, context: S) -> Error
where
    E: StdError + Send + Sync + 'static,
    S: ToString,
{
    Error::other(ErrorWrapper::new(context.to_string(), error))
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

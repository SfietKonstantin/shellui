use colored::Colorize;
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

pub(crate) trait DisplayCli {
    fn to_cli_string(&self) -> String;
    fn display_cli(&self) {
        eprintln!("{}", self.to_cli_string());
    }
}

impl DisplayCli for Error {
    fn to_cli_string(&self) -> String {
        let message = format!("Error: {}", self).red().to_string();

        let source = self.source();
        if let Some(source) = source {
            let caused = "Caused by:".white().dimmed();
            let errors = ErrorIterator::new(Some(source))
                .enumerate()
                .map(|(i, error)| {
                    format!("  ({}) {error}", i + 1)
                        .white()
                        .dimmed()
                        .to_string()
                })
                .collect::<Vec<_>>();
            format!("{message}\n{caused}\n{}", errors.join("\n"))
        } else {
            message
        }
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

struct ErrorIterator<'a> {
    error: Option<&'a (dyn StdError + 'static)>,
}

impl<'a> ErrorIterator<'a> {
    fn new(error: Option<&'a (dyn StdError + 'static)>) -> Self {
        ErrorIterator { error }
    }
}

impl<'a> Iterator for ErrorIterator<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.error {
            let value = self.error;
            self.error = current.source();
            value
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn to_cli_string() {
        env::set_var("NO_COLOR", "1");

        {
            let error = Error::other("Test").to_cli_string();
            assert_eq!(error, "Error: Test")
        }
        {
            let error = with_context(Error::other("Test"), "Failure").to_cli_string();
            assert_eq!(error, "Error: Failure\nCaused by:\n  (1) Test")
        }
        {
            let error = with_context(with_context(Error::other("Error 2"), "Error 1"), "Failure")
                .to_cli_string();
            assert_eq!(
                error,
                "Error: Failure\nCaused by:\n  (1) Error 1\n  (2) Error 2"
            )
        }
    }
}

use inquire::{InquireError, Text};
use std::io::{Error, ErrorKind, Result};

pub trait OrElseQuery {
    type Output;
    fn or_else_query(self, label: &str) -> Result<Self::Output>;
}

impl<T> OrElseQuery for Option<T>
where
    T: ToString,
{
    type Output = String;

    fn or_else_query(self, label: &str) -> Result<Self::Output> {
        match self {
            Some(value) => Ok(value.to_string()),
            None => get_string_input(label),
        }
    }
}

pub fn get_string_input(label: &str) -> Result<String> {
    let name = Text::new(label).prompt();
    match name {
        Ok(value) => Ok(value),
        Err(error) => match error {
            InquireError::NotTTY => Err(Error::other("Not TTY")),
            InquireError::InvalidConfiguration(error) => Err(Error::other(error)),
            InquireError::IO(error) => Err(error),
            InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                Err(Error::new(ErrorKind::Interrupted, "Interrupted"))
            }
            InquireError::Custom(error) => Err(Error::other(error)),
        },
    }
}

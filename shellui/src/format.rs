use crate::errors::{ShellUiError, WithContext};
use colored::Colorize;
use colored_json::to_colored_json_auto;
use serde::Serialize;
pub use shellui_derive::ObjectFormatter;
use std::cmp::max;
use std::error::Error as StdError;
use std::io::{Error, Result};
use std::iter;

pub trait AsFormatted {
    fn unformatted_len(&self) -> usize {
        self.as_unformatted().len()
    }
    fn as_unformatted(&self) -> String;
    fn as_formatted(&self) -> String {
        self.as_unformatted()
    }
    fn print_formatted(&self) {
        eprintln!("{}", self.as_formatted());
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
enum MessageKind {
    #[default]
    Default,
    Info,
    Success,
    Warning,
    Error,
    Hint,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Message {
    kind: MessageKind,
    message: String,
}

impl Message {
    pub fn new<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Default,
            message: value.as_unformatted(),
        }
    }

    pub fn info<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Info,
            message: value.as_unformatted(),
        }
    }

    pub fn success<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Success,
            message: value.as_unformatted(),
        }
    }

    pub fn warning<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Warning,
            message: value.as_unformatted(),
        }
    }

    pub fn error<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Error,
            message: value.as_unformatted(),
        }
    }

    pub fn hint<T>(value: T) -> Self
    where
        T: AsFormatted,
    {
        Message {
            kind: MessageKind::Hint,
            message: value.as_unformatted(),
        }
    }
}

impl AsFormatted for Message {
    fn unformatted_len(&self) -> usize {
        self.message.len()
    }

    fn as_unformatted(&self) -> String {
        self.message.clone()
    }

    fn as_formatted(&self) -> String {
        match &self.kind {
            MessageKind::Default => self.message.clone(),
            MessageKind::Info => self.message.bright_cyan().to_string(),
            MessageKind::Success => self.message.bright_green().to_string(),
            MessageKind::Warning => self.message.bright_yellow().to_string(),
            MessageKind::Error => self.message.bright_red().to_string(),
            MessageKind::Hint => self.message.white().dimmed().to_string(),
        }
    }
}

macro_rules! impl_as_formatted {
    ($ty:ty) => {
        impl AsFormatted for $ty {
            fn as_unformatted(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_as_formatted!(i32);
impl_as_formatted!(i64);
impl_as_formatted!(u32);
impl_as_formatted!(u64);

macro_rules! impl_as_formatted_str {
    ($ty:ty) => {
        impl AsFormatted for $ty {
            fn unformatted_len(&self) -> usize {
                self.len()
            }
            fn as_unformatted(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_as_formatted_str!(String);
impl_as_formatted_str!(&str);

impl AsFormatted for bool {
    fn unformatted_len(&self) -> usize {
        if *self {
            1
        } else {
            0
        }
    }

    fn as_unformatted(&self) -> String {
        if *self {
            "*".to_string()
        } else {
            String::new()
        }
    }
}

impl<T> AsFormatted for Option<T>
where
    T: AsFormatted,
{
    fn as_unformatted(&self) -> String {
        match self {
            Some(value) => value.as_unformatted(),
            None => String::new(),
        }
    }
}

impl AsFormatted for Error {
    fn as_unformatted(&self) -> String {
        self.to_string()
    }

    fn as_formatted(&self) -> String {
        let message = Message::error(self.to_string()).as_formatted();

        let source = self.source();
        if let Some(source) = source {
            let errors = ErrorIterator::new(Some(source))
                .enumerate()
                .map(|(i, error)| Message::hint(format!("  ({}) {error}", i + 1)).as_formatted());

            let errors = iter::once(message)
                .chain(iter::once(Message::hint("Caused by:").as_formatted()))
                .chain(errors)
                .collect::<Vec<_>>();
            errors.join("\n")
        } else {
            message
        }
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

impl AsFormatted for ShellUiError {
    fn as_unformatted(&self) -> String {
        self.to_string()
    }

    fn as_formatted(&self) -> String {
        match self {
            ShellUiError::Error(error) => error.as_formatted(),
            ShellUiError::Warning(warning) => Message::warning(warning).as_formatted(),
            ShellUiError::Interrupt => String::new(),
        }
    }
}

impl<'a, T> AsFormatted for &'a T
where
    T: AsFormatted,
{
    fn unformatted_len(&self) -> usize {
        AsFormatted::unformatted_len(*self)
    }

    fn as_unformatted(&self) -> String {
        AsFormatted::as_unformatted(*self)
    }

    fn as_formatted(&self) -> String {
        AsFormatted::as_formatted(*self)
    }

    fn print_formatted(&self) {
        AsFormatted::print_formatted(*self)
    }
}

pub trait ObjectFormatter {
    type Header: 'static + Clone + AsRef<str>;
    type Mode: 'static + Clone;
    type Output: AsFormatted;

    fn headers(mode: Option<Self::Mode>) -> Vec<Self::Header>;
    fn default_headers() -> Vec<Self::Header> {
        Self::headers(None)
    }
    fn headers_with_mode(mode: Self::Mode) -> Vec<Self::Header> {
        Self::headers(Some(mode))
    }
    fn format_value(&self, mode: Option<Self::Mode>, header: &Self::Header) -> Self::Output;
}

pub trait PrintTable {
    type Item: ObjectFormatter;
    fn format_table(&self, mode: Option<<Self::Item as ObjectFormatter>::Mode>) -> Vec<String>;
    fn print_table(&self, mode: Option<<Self::Item as ObjectFormatter>::Mode>);
    fn print_table_default(&self) {
        self.print_table(None)
    }
    fn print_table_with_mode(&self, mode: <Self::Item as ObjectFormatter>::Mode) {
        self.print_table(Some(mode))
    }
}

impl<T> PrintTable for Vec<T>
where
    T: ObjectFormatter,
{
    type Item = T;

    fn format_table(&self, mode: Option<T::Mode>) -> Vec<String> {
        let headers = T::headers(mode.clone());
        let values = self
            .iter()
            .map(|e| extract_line(e, mode.clone(), &headers))
            .collect::<Vec<_>>();

        let column_count = compute_column_count::<T>(&headers, &values);
        let headers = column_count
            .iter()
            .zip(headers.iter())
            .map(|(size, k)| {
                let header = format!("{:<1$}", k.as_ref(), size);
                header.white().bold().to_string()
            })
            .collect::<Vec<_>>();
        let headers = headers.join("   ");

        iter::once(headers)
            .chain(values.into_iter().map(|line| {
                let line = column_count
                    .iter()
                    .zip(line)
                    .map(|(size, v)| {
                        let formatted = v.as_formatted();
                        let spacing = size - v.unformatted_len() + formatted.len();
                        format!("{:<1$}", formatted, spacing)
                    })
                    .collect::<Vec<_>>();
                line.join("   ")
            }))
            .collect()
    }

    fn print_table(&self, mode: Option<T::Mode>) {
        for line in self.format_table(mode) {
            println!("{line}")
        }
    }
}

fn compute_column_count<T>(headers: &[T::Header], values: &[Vec<T::Output>]) -> Vec<usize>
where
    T: ObjectFormatter,
{
    let zeroes = headers.iter().map(|_| 0).collect::<Vec<_>>();
    let header_sizes = headers
        .iter()
        .map(AsRef::as_ref)
        .map(str::len)
        .collect::<Vec<_>>();
    let value_sizes = values
        .iter()
        .map(|line| line.iter().map(|v| v.unformatted_len()).collect());
    iter::once(header_sizes)
        .chain(value_sizes)
        .fold(zeroes, |prev, current| {
            prev.into_iter()
                .zip(current.iter())
                .map(|(x, y)| max(x, *y))
                .collect()
        })
}

fn extract_line<T>(element: &T, mode: Option<T::Mode>, headers: &[T::Header]) -> Vec<T::Output>
where
    T: ObjectFormatter,
{
    headers
        .iter()
        .map(|k| element.format_value(mode.clone(), k))
        .collect()
}

pub trait PrintSingle {
    type Item: ObjectFormatter;
    fn format_single(&self, mode: Option<<Self::Item as ObjectFormatter>::Mode>) -> Vec<String>;
    fn print_single(&self, mode: Option<<Self::Item as ObjectFormatter>::Mode>);
    fn print_single_default(&self) {
        self.print_single(None)
    }
    fn print_single_with_mode(&self, mode: <Self::Item as ObjectFormatter>::Mode) {
        self.print_single(Some(mode))
    }
}

impl<T> PrintSingle for T
where
    T: ObjectFormatter,
{
    type Item = T;

    fn format_single(&self, mode: Option<T::Mode>) -> Vec<String> {
        let headers = Self::headers(mode.clone());
        let size = headers
            .iter()
            .map(AsRef::as_ref)
            .map(str::len)
            .max()
            .unwrap_or_default();
        headers
            .iter()
            .map(|k| {
                let header = k.as_ref().white().bold();
                let header = format!("{:<1$}", header, size);
                let value = self.format_value(mode.clone(), k);
                format!("{header}   {}", value.as_formatted())
            })
            .collect()
    }

    fn print_single(&self, mode: Option<<Self::Item as ObjectFormatter>::Mode>) {
        for line in self.format_single(mode) {
            println!("{line}")
        }
    }
}

pub trait PrintJson {
    fn print_json(&self) -> Result<()>;
}

impl<T> PrintJson for T
where
    T: Serialize,
{
    fn print_json(&self) -> Result<()> {
        let formatted = to_colored_json_auto(self).with_context("Failed to format to JSON")?;
        println!("{formatted}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    struct TestValue(&'static str, &'static str, &'static str);

    impl ObjectFormatter for TestValue {
        type Header = &'static str;
        type Mode = ();
        type Output = String;

        fn headers(_mode: Option<()>) -> Vec<Self::Header> {
            vec!["id", "label", "a very long header"]
        }

        fn format_value(&self, _mode: Option<()>, header: &Self::Header) -> String {
            match *header {
                "id" => self.0.to_string(),
                "label" => self.1.to_string(),
                "a very long header" => self.2.to_string(),
                _ => String::new(),
            }
        }
    }

    #[test]
    fn test_format_list() {
        env::set_var("NO_COLOR", "1");

        let elements = vec![
            TestValue("1", "label 1", "value"),
            TestValue("a very long id", "l2", "value2"),
        ];
        let table = elements.format_table(None);
        let expected = vec![
            "id               label     a very long header",
            "1                label 1   value             ",
            "a very long id   l2        value2            ",
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_format_single() {
        env::set_var("NO_COLOR", "1");

        let table = TestValue("1", "label 1", "value").format_single(None);
        let expected = vec![
            "id                   1",
            "label                label 1",
            "a very long header   value",
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_format_errors() {
        env::set_var("NO_COLOR", "1");

        {
            let result: Result<()> = Err(Error::other("Test"));
            let error = result.unwrap_err().as_formatted();
            assert_eq!(error, "Test")
        }
        {
            let result: Result<()> = Err(Error::other("Test")).with_context("Failure");
            let error = result.unwrap_err().as_formatted();
            assert_eq!(error, "Failure\nCaused by:\n  (1) Test")
        }
        {
            let result: Result<()> = Err(Error::other("Error 2"))
                .with_context("Error 1")
                .with_context("Failure");
            let error = result.unwrap_err().as_formatted();
            assert_eq!(error, "Failure\nCaused by:\n  (1) Error 1\n  (2) Error 2")
        }
    }
}

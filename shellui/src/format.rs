use crate::errors::WithContext;
use colored::Colorize;
use colored_json::to_colored_json_auto;
use serde::Serialize;
pub use shellui_derive::ObjectFormatter;
use std::cmp::max;
use std::io::Result;
use std::iter;

pub trait ObjectFormatter {
    type Header: 'static + Clone + AsRef<str>;
    type Mode: 'static + Clone;

    fn headers(mode: Option<Self::Mode>) -> Vec<Self::Header>;
    fn default_headers() -> Vec<Self::Header> {
        Self::headers(None)
    }
    fn headers_with_mode(mode: Self::Mode) -> Vec<Self::Header> {
        Self::headers(Some(mode))
    }
    fn format_value(&self, mode: Option<Self::Mode>, header: &Self::Header) -> String;
}

pub trait FormatField {
    fn format_field(&self) -> String;
}
macro_rules! impl_format_field {
    ($ty:ty) => {
        impl FormatField for $ty {
            fn format_field(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_format_field!(i32);
impl_format_field!(i64);
impl_format_field!(u32);
impl_format_field!(u64);
impl_format_field!(String);
impl_format_field!(&str);

impl FormatField for bool {
    fn format_field(&self) -> String {
        if *self {
            "*".to_string()
        } else {
            String::new()
        }
    }
}

impl<T> FormatField for Option<T>
where
    T: FormatField,
{
    fn format_field(&self) -> String {
        match self {
            Some(value) => value.format_field(),
            None => String::new(),
        }
    }
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

        let column_count = compute_column_count(&headers, &values);
        let headers = column_count
            .iter()
            .zip(headers.iter())
            .map(|(size, k)| {
                let header = format!("{:<1$}", k.as_ref(), size);
                header.white().bold().to_string()
            })
            .collect::<Vec<_>>();
        let headers = headers.join(" ");

        iter::once(headers)
            .chain(values.into_iter().map(|line| {
                let line = column_count
                    .iter()
                    .zip(line)
                    .map(|(size, v)| {
                        let value = format!("{:<1$}", v, size);
                        value.white().to_string()
                    })
                    .collect::<Vec<_>>();
                line.join(" ")
            }))
            .collect()
    }

    fn print_table(&self, mode: Option<T::Mode>) {
        for line in self.format_table(mode) {
            println!("{line}")
        }
    }
}

fn compute_column_count<K>(headers: &[K], values: &[Vec<String>]) -> Vec<usize>
where
    K: AsRef<str>,
{
    let zeroes = headers.iter().map(|_| 0).collect::<Vec<_>>();
    let header_sizes = headers
        .iter()
        .map(AsRef::as_ref)
        .map(str::len)
        .collect::<Vec<_>>();
    let value_sizes = values
        .iter()
        .map(|line| line.iter().map(|v| v.len()).collect());
    iter::once(header_sizes)
        .chain(value_sizes)
        .fold(zeroes, |prev, current| {
            prev.into_iter()
                .zip(current.iter())
                .map(|(x, y)| max(x, *y))
                .collect()
        })
}

fn extract_line<T>(element: &T, mode: Option<T::Mode>, headers: &[T::Header]) -> Vec<String>
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
                format!("{header} {value}")
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
            "id             label   a very long header",
            "1              label 1 value             ",
            "a very long id l2      value2            ",
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_format_single() {
        env::set_var("NO_COLOR", "1");

        let table = TestValue("1", "label 1", "value").format_single(None);
        let expected = vec![
            "id                 1",
            "label              label 1",
            "a very long header value",
        ];
        assert_eq!(table, expected);
    }
}

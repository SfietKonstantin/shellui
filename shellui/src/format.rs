use crate::errors::WithContext;
use colored::Colorize;
use colored_json::to_colored_json_auto;
use serde::Serialize;
use std::cmp::max;
use std::io::Result;
use std::iter;

pub trait ObjectFormatter {
    type Header: 'static + Clone + AsRef<str>;
    fn headers() -> &'static [Self::Header];
    fn format_value(&self, header: &Self::Header) -> String;
}

fn format_list<T>(elements: &[T], headers: &[T::Header]) -> Vec<String>
where
    T: ObjectFormatter,
{
    let values = elements
        .iter()
        .map(|e| extract_line(e, headers))
        .collect::<Vec<_>>();

    let column_count = compute_column_count(headers, &values);
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

pub fn print_list<T>(elements: &[T])
where
    T: ObjectFormatter,
{
    let headers = T::headers();
    for line in format_list(elements, headers) {
        println!("{line}")
    }
}

fn format_single<T>(element: &T, headers: &[T::Header]) -> Vec<String>
where
    T: ObjectFormatter,
{
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
            let value = element.format_value(k);
            format!("{header} {value}")
        })
        .collect()
}

pub fn print_single<T>(element: &T)
where
    T: ObjectFormatter,
{
    let headers = T::headers();
    for line in format_single(element, headers) {
        println!("{line}")
    }
}

pub fn print_json<T>(element: &T) -> Result<()>
where
    T: Serialize,
{
    let formatted = to_colored_json_auto(element).with_context("Failed to format to JSON")?;
    println!("{formatted}");
    Ok(())
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
                .zip(current)
                .map(|(x, y)| max(x, y))
                .collect()
        })
}

fn extract_line<T>(element: &T, headers: &[T::Header]) -> Vec<String>
where
    T: ObjectFormatter,
{
    headers.iter().map(|k| element.format_value(k)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    struct TestValue(&'static str, &'static str, &'static str);

    impl ObjectFormatter for TestValue {
        type Header = &'static str;

        fn headers() -> &'static [Self::Header] {
            &["id", "label", "a very long header"]
        }

        fn format_value(&self, header: &Self::Header) -> String {
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
        let table = format_list(&elements, TestValue::headers());
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

        let table = format_single(&TestValue("1", "label 1", "value"), TestValue::headers());
        let expected = vec![
            "id                 1",
            "label              label 1",
            "a very long header value",
        ];
        assert_eq!(table, expected);
    }
}

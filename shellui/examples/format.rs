use shellui::errors::WithContext;
use shellui::format::{AsFormatted, Message, PrintSingle, PrintTable};
use shellui_derive::ObjectFormatter;
use std::io::{Error, Result};

#[derive(ObjectFormatter)]
struct Simple {
    #[object_formatter(header = "Id")]
    id: String,
    #[object_formatter(header = "Status", level = "info")]
    status: String,
    #[object_formatter(header = "Value", with = "format_value")]
    value: i32,
}

impl Simple {
    pub fn new(id: String, status: String, value: i32) -> Self {
        Self { id, status, value }
    }
}

fn format_value(value: &i32) -> Message {
    if *value > 50 {
        Message::warning(value)
    } else {
        Message::success(value)
    }
}

fn main() {
    // Messages
    Message::new("This is the default format").print_formatted();
    Message::info("This is an info format").print_formatted();
    Message::success("This is a success format").print_formatted();
    Message::warning("This is a warning format").print_formatted();
    Message::error("This is an error format").print_formatted();
    Message::hint("This is a hint format").print_formatted();
    eprintln!();

    // Table
    vec![
        Simple::new("id1".to_string(), "success".to_string(), 25),
        Simple::new("id2".to_string(), "error".to_string(), 75),
    ]
    .print_table_default();
    eprintln!();

    // Single element
    Simple::new("id3".to_string(), "success".to_string(), 30).print_single_default();
    eprintln!();

    // Errors
    let result: Result<()> = Err(Error::other("Technical error"))
        .with_context("Failed to perform operation")
        .with_context("Error: could not execute command");
    result.unwrap_err().print_formatted();
}

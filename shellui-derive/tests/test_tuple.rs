use shellui::format::ObjectFormatter;

#[derive(ObjectFormatter)]
struct Tuple(
    #[object_formatter(header = "Id")] String,
    #[object_formatter(header = "Label")] String,
);

#[test]
fn test() {}

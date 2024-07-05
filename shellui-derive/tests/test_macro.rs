use shellui::format::ObjectFormatter;

#[derive(ObjectFormatter)]
struct Simple {
    #[object_formatter(header = "Id")]
    id: String,
    #[object_formatter(header = "Label")]
    label: String,
    #[object_formatter(inline)]
    coordinates: Coordinates,
    #[object_formatter(header = "Value", mode = "special")]
    value: i32,
    _ignored: bool,
}

impl Simple {
    pub fn new(id: String, label: String, coordinates: Coordinates, value: i32) -> Self {
        Self {
            id,
            label,
            coordinates,
            value,
            _ignored: true,
        }
    }
}

#[derive(ObjectFormatter)]
struct Coordinates {
    #[object_formatter(header = "Host")]
    host: String,
    #[object_formatter(header = "Port")]
    port: u32,
}
impl Coordinates {
    pub fn new(host: String, port: u32) -> Self {
        Self { host, port }
    }
}

#[derive(ObjectFormatter)]
struct NoField {
    _field1: String,
    _field2: String,
    _field3: String,
}
#[derive(ObjectFormatter)]
struct Tuple(
    #[object_formatter(header = "Id")] String,
    #[object_formatter(header = "Label")] String,
);

#[derive(ObjectFormatter)]
struct Unit;

#[test]
fn test_derive() {
    let headers = vec![
        "Id".to_string(),
        "Label".to_string(),
        "Host".to_string(),
        "Port".to_string(),
    ];
    assert_eq!(Simple::default_headers(), headers);
    let headers_with_mode = vec![
        "Id".to_string(),
        "Label".to_string(),
        "Host".to_string(),
        "Port".to_string(),
        "Value".to_string(),
    ];
    assert_eq!(Simple::headers_with_mode("special"), headers_with_mode);

    let value = Simple::new(
        "id".to_string(),
        "label".to_string(),
        Coordinates::new("http://localhost".to_string(), 8888),
        123,
    );
    assert_eq!(value.format_value(None, &"Id"), "id".to_string());
    assert_eq!(value.format_value(None, &"Label"), "label".to_string());
    assert_eq!(
        value.format_value(None, &"Host"),
        "http://localhost".to_string()
    );
    assert_eq!(value.format_value(None, &"Port"), "8888".to_string());
    assert_eq!(value.format_value(None, &"Value"), "123".to_string());
}

#[test]
fn test_derive_tuple() {
    let headers = vec!["Id".to_string(), "Label".to_string()];
    assert_eq!(Tuple::default_headers(), headers);

    let value = Tuple("id".to_string(), "label".to_string());
    assert_eq!(value.format_value(None, &"Id"), "id".to_string());
    assert_eq!(value.format_value(None, &"Label"), "label".to_string());
}

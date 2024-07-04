use shellui::format::ObjectFormatter;

#[derive(ObjectFormatter)]
struct Simple {
    #[header("Id")]
    id: String,
    #[header("Label")]
    label: String,
    #[header(inline)]
    coordinates: Coordinates,
    #[header("Value")]
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
    #[header("Host")]
    host: String,
    #[header("Port")]
    port: u32,
}
impl Coordinates {
    pub fn new(host: String, port: u32) -> Self {
        Self { host, port }
    }
}

#[test]
fn test_derive() {
    let headers = vec![
        "Id".to_string(),
        "Label".to_string(),
        "Host".to_string(),
        "Port".to_string(),
        "Value".to_string(),
    ];
    assert_eq!(Simple::headers(), headers);

    let simple = Simple::new(
        "id".to_string(),
        "label".to_string(),
        Coordinates::new("http://localhost".to_string(), 8888),
        123,
    );
    assert_eq!(simple.format_value(&"Id"), "id".to_string());
    assert_eq!(simple.format_value(&"Label"), "label".to_string());
    assert_eq!(simple.format_value(&"Host"), "http://localhost".to_string());
    assert_eq!(simple.format_value(&"Port"), "8888".to_string());
    assert_eq!(simple.format_value(&"Value"), "123".to_string());
}

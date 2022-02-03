use crate::parser::types::JsonType;

#[derive(Debug, Eq, PartialEq)]
struct JsonSchema {
    fields: Vec<JsonType>
}

impl JsonSchema {
    fn new(file: String) -> Self {
        Self {
            fields: vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::schema::JsonSchema;
    use crate::parser::types::JsonType;

    #[test]
    fn basic_types() {
        let json_string = "\
{
    \"field\": 54,
    \"field2\": \"Hola\",
    \"field3\": 12.23,
    \"filed4\": true,
}
        ";

        let expected_result = JsonSchema {
            fields: vec![
                JsonType::Int(String::from("field")),
                JsonType::String(String::from("field2")),
                JsonType::Float(String::from("field3")),
                JsonType::Bool(String::from("field4"))
            ]
        };

        let schema = JsonSchema::new(json_string.to_owned());

        assert_eq!(schema, expected_result);
    }
}
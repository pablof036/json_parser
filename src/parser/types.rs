
/// Holds the possible types of a JSON object, with a String as field name
#[derive(Debug, Eq, PartialEq)]
pub enum JsonType {
    Int(String),
    Float(String),
    String(String),
    Bool(String),
    JsonObject(String, Vec<JsonType>),
    JsonArray(String, JsonArrayType)
}

/// Holds the possible types of a Json array (no field name).
#[derive(Debug, Eq, PartialEq)]
pub enum JsonArrayType {
    Int,
    Float,
    String,
    Bool,
    JsonObject(Vec<JsonType>),
    JsonArray(Box<JsonType>)
}
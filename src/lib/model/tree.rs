/// Holds the possible types of a JSON object, with a String as field name
#[derive(Debug, Eq, PartialEq)]
pub enum JsonTree {
    Int(String),
    Float(String),
    String(String),
    Bool(String),
    JsonObject(String, Vec<JsonTree>),
    JsonArray(String, JsonArrayType),
}

/// Holds the possible types of a Json array (no field name).
#[derive(Debug, Eq, PartialEq)]
pub enum JsonArrayType {
    Int,
    Float,
    String,
    Bool,
    JsonObject(Vec<JsonTree>),
    JsonArray(Box<JsonArrayType>)
}
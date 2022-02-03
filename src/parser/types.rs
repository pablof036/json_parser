#[derive(Debug, Eq, PartialEq)]
pub enum JsonType {
    Int(String),
    Float(String),
    String(String),
    Bool(String),
    JsonObject(String, Vec<JsonType>),
    JsonArray(String, JsonArrayType)
}

#[derive(Debug, Eq, PartialEq)]
pub enum JsonArrayType {
    Int,
    Float,
    String,
    Bool,
    JsonObject(Vec<JsonType>),
    JsonArray(Box<JsonType>)
}
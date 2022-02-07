/// Holds the possible types of a JSON object, with a String as field name
#[derive(Debug, Eq, PartialEq)]
pub enum JsonTree<'a> {
    Int(&'a str),
    Float(&'a str),
    String(&'a str),
    Bool(&'a str),
    JsonObject(&'a str, Vec<JsonTree<'a>>),
    JsonArray(&'a str, JsonArrayType<'a>),
    Root(Vec<JsonTree<'a>>)
}

/// Holds the possible types of a Json array (no field name).
#[derive(Debug, Eq, PartialEq)]
pub enum JsonArrayType<'a> {
    Int,
    Float,
    String,
    Bool,
    JsonObject(Vec<JsonTree<'a>>),
    JsonArray(Box<JsonArrayType<'a>>)
}
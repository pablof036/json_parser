#[derive(Debug, Eq, PartialEq)]
pub enum JsonToken {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Name(String),
    Value(JsonType),
}

#[derive(Debug, Eq, PartialEq)]
pub enum JsonType {
    Int,
    Float,
    Bool,
    String,
    Null
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub line: usize,
    pub col: usize,
    pub value: JsonToken,
}

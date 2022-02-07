#[derive(Debug, Eq, PartialEq)]
pub enum JsonToken<'a> {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Name(&'a str),
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
pub struct Token<'a> {
    pub line: usize,
    pub col: usize,
    pub value: JsonToken<'a>,
}

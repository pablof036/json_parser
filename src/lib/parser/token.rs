use std::cell::RefCell;
use std::iter::{Enumerate, Peekable};
use std::rc::Rc;
use std::vec::IntoIter;
use crate::lib::parser::types::{JsonArrayType, JsonTree};
use thiserror::Error;
use crate::lib::parser::token::TokenizerError::SyntaxError;
use crate::lib::parser::types::JsonArrayType::JsonObject;

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
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token<'a> {
    pub line: usize,
    pub col: usize,
    pub value: JsonToken<'a>,
}

pub struct Tree<'a, T> {
    current: Rc<RefCell<&'a mut Vec<T>>>,
    parents: Vec<Rc<RefCell<&'a mut Vec<T>>>>
}

#[derive(Error, Debug)]
enum TokenizerError {
    #[error("syntax error detected near line {0} column {1}")]
    SyntaxError(usize, usize),
    #[error("unknown syntax error")]
    UnknownSyntaxError,
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    token_iter: Peekable<Enumerate<IntoIter<Token<'a>>>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            token_iter: tokens.into_iter().enumerate().peekable(),
        }
    }

    fn parse_new_array_type(old_type: Option<JsonArrayType<'a>>, new_type: JsonArrayType<'a>, line: usize, col: usize) -> Result<JsonArrayType<'a>, TokenizerError> {
        if let Some(old_type) = old_type {
            if old_type == new_type {
                return Ok(new_type);
            }

            if let JsonArrayType::JsonObject(mut old_tree) = old_type {
                if let JsonArrayType::JsonObject(mut new_tree) = new_type {
                    new_tree.into_iter().for_each(|json_type| {
                       if !old_tree.contains(&json_type) {
                           old_tree.push(json_type)
                       }
                    });

                    return Ok(JsonArrayType::JsonObject(old_tree));
                }

                return Err(SyntaxError(line, col));
            }

            return Err(TokenizerError::SyntaxError(line, col));
        }

        Ok(new_type)
    }

    fn parse_array_token(&mut self, name: &'a str) -> Result<JsonTree<'a>, TokenizerError> {
        let mut array_type = None;

        while let Some((i, token)) = self.token_iter.next() {
            match token.value {
                JsonToken::ArrayEnd => {
                    if let Some(array_type) = array_type {
                        return Ok(JsonTree::JsonArray(name, array_type));
                    }

                    return Err(TokenizerError::SyntaxError(token.line, token.col));
                },
                JsonToken::ArrayStart => {
                    let deeper_array = self.parse_array_token("")?;
                    if let JsonTree::JsonArray(_, deeper_array_type) = deeper_array {
                        let deeper_array_type = JsonArrayType::JsonArray(Box::new(deeper_array_type));
                        array_type = Some(Self::parse_new_array_type(array_type, deeper_array_type, token.line, token.col)?);
                    } else {
                        return Err(TokenizerError::UnknownSyntaxError);
                    }
                }
                JsonToken::ObjectStart => {
                    let object = self.parse_object_token()?;
                    let new_type = JsonArrayType::JsonObject(object);
                    array_type = Some(Self::parse_new_array_type(array_type, new_type, token.line, token.col)?);
                },
                JsonToken::Value(json_type) => {
                    let value_type;
                    match json_type {
                        JsonType::Int => value_type = JsonArrayType::Int,
                        JsonType::Float => value_type = JsonArrayType::Float,
                        JsonType::Bool => value_type = JsonArrayType::Bool,
                        JsonType::String => value_type = JsonArrayType::String,
                    }
                    array_type = Some(Self::parse_new_array_type(array_type, value_type, token.line, token.col)?);
                },
                JsonToken::Comma => (),
                _ => {
                    return Err(TokenizerError::SyntaxError(token.line, token.col));
                }
            }
        }

        if let Some(array_type) = array_type {
            Ok(JsonTree::JsonArray(name, array_type))
        } else {
            Err(TokenizerError::UnknownSyntaxError)
        }
    }

    fn parse_object_token(&mut self) -> Result<Vec<JsonTree<'a>>, TokenizerError> {
        let mut object = Vec::new();
        let mut name = None;
        let mut actual_count = 0;
        while let Some((_, token)) = self.token_iter.next() {
            match token.value {
                JsonToken::ObjectStart => {
                    if actual_count != 0 {
                        if let Some(name) = name {
                            let deeper_object = self.parse_object_token()?;
                            object.push(JsonTree::JsonObject(name, deeper_object));
                        } else {
                            return Err(TokenizerError::SyntaxError(token.line, token.col));
                        }
                        name = None;
                    }
                }
                JsonToken::ObjectEnd => {
                    return Ok(object);
                }
                JsonToken::ArrayStart => {
                    if let Some(name) = name {
                        let array = self.parse_array_token(name)?;
                        object.push(array)
                    } else {
                        return Err(TokenizerError::SyntaxError(token.line, token.col));
                    }

                    name = None;
                }
                JsonToken::ArrayEnd => {}
                JsonToken::Colon => {
                    if name.is_none() {
                        return Err(TokenizerError::SyntaxError(token.line, token.col));
                    }
                }
                JsonToken::Comma => {}
                JsonToken::Name(field_name) => {
                    if name.is_some() {
                        return Err(TokenizerError::SyntaxError(token.line, token.col));
                    }

                    name = Some(field_name);
                }
                JsonToken::Value(value_type) => {
                    if let Some(name) = name {
                        match value_type {
                            JsonType::Int => object.push(JsonTree::Int(name)),
                            JsonType::Float => object.push(JsonTree::Float(name)),
                            JsonType::Bool => object.push(JsonTree::Bool(name)),
                            JsonType::String => object.push(JsonTree::String(name)),
                        }
                    } else {
                        return Err(TokenizerError::SyntaxError(token.line, token.col));
                    }

                    name = None;
                }
            }

            actual_count += 1;
        }
        Ok(object)
    }

    pub fn start_tokenizer(mut self) -> anyhow::Result<JsonTree<'a>> {
        Ok(JsonTree::Root(self.parse_object_token()?))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::Lexer;
    use crate::parser::token::{JsonType, Tokenizer};
    use crate::parser::types::{JsonArrayType, JsonTree};

    #[test]
    #[should_panic]
    fn syntax_error_on_no_root_brace() {
        let json = "\"error\": \"oof\"";

        let lexer = Lexer::new(json);
        let tokenizer = Tokenizer::new(lexer.start_lex());
        tokenizer.start_tokenizer().unwrap();
    }

    #[test]
    fn simple_json() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": 45.3, \"f4\": 12}";
        let expected_result = JsonTree::Root(vec![
            JsonTree::String("f1"),
            JsonTree::Bool("f2"),
            JsonTree::Float("f3"),
            JsonTree::Int("f4"),
        ]);

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);

        let tree = tokenizer.start_tokenizer().unwrap();
        assert_eq!(tree, expected_result);
    }

    #[test]
    fn nested_json_object() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": { \"f4\": 45.3, \"f5\": {\"f6\": true, \"f7\":\"aº\"}}, \"a\": 32}";

        let expected_result = JsonTree::Root(
            vec![
                JsonTree::String("f1"),
                JsonTree::Bool("f2"),
                JsonTree::JsonObject("f3", vec![
                    JsonTree::Float("f4"),
                    JsonTree::JsonObject("f5", vec![
                        JsonTree::Bool("f6"),
                        JsonTree::String("f7"),
                    ]),
                ]),
                JsonTree::Int("a")
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

    #[test]
    fn simple_array() {
        let json = "{\"f1\": [5, 3, 2, 1]}";

        let expected_result = JsonTree::Root(
            vec![
                JsonTree::JsonArray("f1", JsonArrayType::Int)
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }


    #[test]
    fn nested_array() {
        let json = "{\"f1\": [[5, 3], [2, 1]]}";

        let expected_result = JsonTree::Root(
            vec![
                JsonTree::JsonArray("f1", JsonArrayType::JsonArray(Box::new(JsonArrayType::Int)))
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

    #[test]
    #[should_panic]
    fn different_nested_array_error() {
        let json = "{\"f1\": [[5, 3], [2.0, 1.0]]}";

        let expected_result = JsonTree::Root(
            vec![
                JsonTree::JsonArray("f1", JsonArrayType::JsonArray(Box::new(JsonArrayType::Int)))
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        tokenizer.start_tokenizer().unwrap();
    }

    #[test]
    fn array_with_object() {
        let json = "{\"f1\": [{\"f2\": 432, \"f3\": true}]}";

        let expected_result = JsonTree::Root(
            vec![
                JsonTree::JsonArray("f1", JsonArrayType::JsonObject(
                    vec![
                        JsonTree::Int("f2"),
                        JsonTree::Bool("f3")
                    ]
                ))
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

    #[test]
    fn array_object_adding() {
        let json = "{\"f1\": [{\"f2\": 432, \"f3\": true}, {\"f4\": 43.2}]}";
        let expected_result = JsonTree::Root(
            vec![
                JsonTree::JsonArray("f1", JsonArrayType::JsonObject(
                    vec![
                        JsonTree::Int("f2"),
                        JsonTree::Bool("f3"),
                        JsonTree::Float("f4")
                    ]
                ))
            ]
        );

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

}
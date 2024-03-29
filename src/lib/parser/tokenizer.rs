use std::iter::{Enumerate, Peekable};
use std::vec::IntoIter;
use crate::lib::model::tree::{JsonArrayType, JsonTree};
use thiserror::Error;
use crate::lib::model::token::{JsonToken, JsonType, Token};
use crate::lib::parser::tokenizer::TokenizerError::{NullNotSupportedError, SyntaxError};

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("syntax error detected near line {} column {1}", .0 + 1)]
    SyntaxError(usize, usize),
    #[error("unknown syntax error")]
    UnknownSyntaxError,
    #[error("null values are not supported. Near line {} column {1}", .0 + 1)]
    NullNotSupportedError(usize, usize),
    #[error("empty arrays are not supported. Near line {} column {1}", .0 + 1)]
    EmptyArrayNotSupportedError(usize, usize),
}

#[derive(Debug)]
pub struct Tokenizer {
    token_iter: Peekable<Enumerate<IntoIter<Token>>>,
}

impl Tokenizer {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            token_iter: tokens.into_iter().enumerate().peekable(),
        }
    }

    /// Parses a new array, if the array's type is an object, it will join the object's fields.
    /// # Arguments
    /// * `old_type` previous array, if it's an object, its field will be joined with those of the new type.
    /// * `new_type` new array type
    /// # Returns
    /// New array type
    /// # Errors
    /// If the old type is not the same as the new type, an error will be returned.
    fn parse_new_array_type(old_type: Option<JsonArrayType>, new_type: JsonArrayType, line: usize, col: usize) -> Result<JsonArrayType, TokenizerError> {
        if let Some(old_type) = old_type {
            if old_type == new_type {
                return Ok(new_type);
            }

            if let JsonArrayType::JsonObject(mut old_tree) = old_type {
                if let JsonArrayType::JsonObject(new_tree) = new_type {
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

    /// Parses an array token
    /// # Arguments
    /// * `name` name of the array's field
    fn parse_array_token(&mut self, name: String) -> Result<JsonTree, TokenizerError> {
        let mut array_type = None;

        while let Some((_, token)) = self.token_iter.next() {
            match token.value {
                JsonToken::ArrayEnd => {
                    if let Some(array_type) = array_type {
                        return Ok(JsonTree::JsonArray(name, array_type));
                    }

                    return Err(TokenizerError::EmptyArrayNotSupportedError(token.line, token.col));
                }
                JsonToken::ArrayStart => {
                    let deeper_array = self.parse_array_token(String::new())?;
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
                }
                JsonToken::Value(json_type) => {
                    let value_type;
                    match json_type {
                        JsonType::Int => value_type = JsonArrayType::Int,
                        JsonType::Float => value_type = JsonArrayType::Float,
                        JsonType::Bool => value_type = JsonArrayType::Bool,
                        JsonType::String => value_type = JsonArrayType::String,
                        JsonType::Null => return Err(NullNotSupportedError(token.line, token.col)),
                    }
                    array_type = Some(Self::parse_new_array_type(array_type, value_type, token.line, token.col)?);
                }
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

    /// Parses a list of [JsonToken]
    /// # Returns
    /// Object's fields
    /// # Errors
    /// If a syntax error is found, a [TokenizerError] will be returned.
    fn parse_object_token(&mut self) -> Result<Vec<JsonTree>, TokenizerError> {
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
                            JsonType::Null => return Err(TokenizerError::NullNotSupportedError(token.line, token.col))
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

    /// Starts the conversion from the list of tokens to a [JsonTree].
    /// # Returns
    /// JSON representation in list of [JsonTree]
    pub fn start_tokenizer(mut self) -> Result<Vec<JsonTree>, TokenizerError> {
        Ok(self.parse_object_token()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::parser::lexer::Lexer;
    use crate::lib::parser::tokenizer::Tokenizer;
    use crate::lib::model::tree::{JsonArrayType, JsonTree};

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
        let expected_result = vec![
            JsonTree::String("f1".to_owned()),
            JsonTree::Bool("f2".to_owned()),
            JsonTree::Float("f3".to_owned()),
            JsonTree::Int("f4".to_owned()),
        ];

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);

        let tree = tokenizer.start_tokenizer().unwrap();
        assert_eq!(tree, expected_result);
    }

    #[test]
    fn nested_json_object() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": { \"f4\": 45.3, \"f5\": {\"f6\": true, \"f7\":\"aº\"}}, \"a\": 32}";

        let expected_result = vec![
            JsonTree::String("f1".to_owned()),
            JsonTree::Bool("f2".to_owned()),
            JsonTree::JsonObject("f3".to_owned(), vec![
                JsonTree::Float("f4".to_owned()),
                JsonTree::JsonObject("f5".to_owned(), vec![
                    JsonTree::Bool("f6".to_owned()),
                    JsonTree::String("f7".to_owned()),
                ]),
            ]),
            JsonTree::Int("a".to_owned()),
        ];

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

    #[test]
    fn simple_array() {
        let json = "{\"f1\": [5, 3, 2, 1]}";

        let expected_result = vec![
            JsonTree::JsonArray("f1".to_owned(), JsonArrayType::Int)
        ];

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }


    #[test]
    fn nested_array() {
        let json = "{\"f1\": [[5, 3], [2, 1]]}";

        let expected_result = vec![
            JsonTree::JsonArray("f1".to_owned(), JsonArrayType::JsonArray(Box::new(JsonArrayType::Int)))
        ];


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

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        tokenizer.start_tokenizer().unwrap();
    }

    #[test]
    fn array_with_object() {
        let json = "{\"f1\": [{\"f2\": 432, \"f3\": true}]}";

        let expected_result = vec![
            JsonTree::JsonArray("f1".to_owned(), JsonArrayType::JsonObject(
                vec![
                    JsonTree::Int("f2".to_owned()),
                    JsonTree::Bool("f3".to_owned()),
                ]
            ))
        ];

        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }

    #[test]
    fn array_object_adding() {
        let json = "{\"f1\": [{\"f2\": 432, \"f3\": true}, {\"f4\": 43.2}]}";
        let expected_result = vec![
            JsonTree::JsonArray("f1".to_owned(), JsonArrayType::JsonObject(
                vec![
                    JsonTree::Int("f2".to_owned()),
                    JsonTree::Bool("f3".to_owned()),
                    JsonTree::Float("f4".to_owned()),
                ]
            ))
        ];


        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        let tree = tokenizer.start_tokenizer().unwrap();

        assert_eq!(tree, expected_result);
    }


    #[test]
    #[should_panic(expected = "null values are not supported")]
    fn fail_on_null() {
        let json = "{ \"f2\": null }";
        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        tokenizer.start_tokenizer().unwrap();
    }

    #[test]
    #[should_panic(expected = "empty arrays are not supported")]
    fn fail_on_empty_array() {
        let json = "{ \"f2\": [] }";
        let lexer = Lexer::new(json);
        let lexer_result = lexer.start_lex();
        let tokenizer = Tokenizer::new(lexer_result);
        tokenizer.start_tokenizer().unwrap();
    }
}
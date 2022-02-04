use std::iter::{Enumerate, Peekable};
use std::vec::IntoIter;
use crate::parser::types::JsonTree;
use thiserror::Error;
use crate::parser::token::TokenizerError::SyntaxError;

#[derive(Debug, Eq, PartialEq)]
pub enum JsonToken<'a> {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Name(&'a str),
    Value(JsonType)
}

#[derive(Debug, Eq, PartialEq)]
pub enum JsonType {
    Int,
    Float,
    Bool,
    String,
    Object,
    Array
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token<'a> {
    pub line: usize,
    pub col: usize,
    pub value: JsonToken<'a>
}

#[derive(Error, Debug)]
enum TokenizerError {
    #[error("syntax error detected near line {0} column {1}")]
    SyntaxError(usize, usize)
}

enum TokenizerNextStep {
    ParseName,
    ParseObject,
    Done
}

enum TokenClosureNextStep {
    Advance,
    Done,
    Error(TokenizerError)
}

#[derive(Debug)]
struct Tokenizer<'a> {
    token_iter: Peekable<Enumerate<IntoIter<Token<'a>>>>,
    tree: JsonTree<'a>,
}

impl<'a> Tokenizer<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            token_iter: tokens.into_iter().enumerate().peekable(),
            tree: JsonTree::Root(vec![]),
        }
    }

    fn parse_object_token(&mut self) -> Result<(), TokenizerError> {
        if let JsonTree::Root(tree) = &mut self.tree {
            let mut field_name = None;

            while let Some((i, token)) = &self.token_iter.next() {
                if let Some((y, next_token)) = self.token_iter.peek() {
                    match token.value {
                        JsonToken::Name(name) => {
                            if next_token.value != JsonToken::Colon {
                                return Err(SyntaxError(token.line, token.col))
                            }

                            field_name = Some(name);
                        },
                        JsonToken::Colon => {
                            if let Some(field_name) = field_name {
                                match next_token.value {
                                    JsonToken::Value(JsonType::Int) => tree.push(JsonTree::Int(field_name)),
                                    JsonToken::Value(JsonType::Float) => tree.push(JsonTree::Float(field_name)),
                                    JsonToken::Value(JsonType::Bool) => tree.push(JsonTree::Bool(field_name)),
                                    JsonToken::Value(JsonType::String) => tree.push(JsonTree::String(field_name)),
                                    _ => (),
                                }
                            } else {
                                return Err(SyntaxError(token.line, token.col));
                            }

                            field_name = None;
                        }
                        JsonToken::ObjectEnd => {
                            break
                        },
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_name_token(&mut self) -> Result<(), TokenizerError> {
        if let Some((i, next_token)) = self.token_iter.peek() {
            if next_token.value != JsonToken::Colon {
                return Err(TokenizerError::SyntaxError(next_token.line, next_token.col));
            }


        }


        Ok(())
    }

    fn tokenize(&mut self) -> Result<TokenizerNextStep, TokenizerError> {

        while let Some((i, token)) = self.token_iter.peek() {
            if i == &0 && token.value != JsonToken::ObjectStart {
                return Err(SyntaxError(token.line, *i))
            }

            match token.value {
                JsonToken::ObjectStart => {
                    self.token_iter.next();
                    return Ok(TokenizerNextStep::ParseObject)
                },
                JsonToken::ObjectEnd => (),
                JsonToken::ArrayStart => (),
                JsonToken::ArrayEnd => (),
                JsonToken::Colon => (),
                JsonToken::Comma => (),
                JsonToken::Name(name) => (),
                JsonToken::Value(JsonType::Int) => (),
                JsonToken::Value(JsonType::Float) => (),
                JsonToken::Value(JsonType::Bool) => (),
                JsonToken::Value(JsonType::String) => (),
                JsonToken::Value(JsonType::Object) => (),
                JsonToken::Value(JsonType::Array) => (),
            }
        }

        Ok(TokenizerNextStep::Done)
    }

    fn start_tokenizer(mut self) -> anyhow::Result<JsonTree<'a>> {
        self.parse_object_token()?;
        Ok(self.tree)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::Lexer;
    use crate::parser::token::{Tokenizer, TokenizerError};
    use crate::parser::types::JsonTree;

    #[test]
    #[should_panic]
    fn syntax_error_on_no_root_brace()  {
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
}
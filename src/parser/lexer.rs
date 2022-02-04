use std::iter::{Enumerate, Peekable};
use std::str::{Chars, Lines};
use crate::parser::lexer::NextStep::{Done, LexCharacter};
use crate::parser::token::{JsonToken, JsonType, Token};

#[derive(Debug, PartialEq, Eq)]
enum NextStep {
    LexNumberType,
    LexCharacter,
    LexName,
    LexString,
    LexBoolean,
    Done
}

#[derive(Debug, PartialEq, Eq)]
enum NextLexStep {
    Done,
    Advance,
    Skip
}

struct Lexer<'a> {
    json: &'a str,
    lines: Enumerate<Lines<'a>>,
    current_line: usize,
    char_iter: Option<Peekable<Enumerate<Chars<'a>>>>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(json: &'a str) -> Self {
        let lines = json.lines().enumerate();
        let char_iter = json.chars().enumerate();
        Self {
            json,
            lines,
            current_line: 0,
            char_iter: None,
            tokens: vec![],
        }
    }

    fn lex_character(&mut self) -> NextStep {
        if let Some(char_iter) = &mut self.char_iter {
            while let Some((i, char)) = char_iter.next() {
                match char {
                    '{' => self.tokens.push(Token {
                        value: JsonToken::ObjectStart,
                        col: i,
                        line: self.current_line,
                    }),
                    '}' => self.tokens.push(Token {
                        value: JsonToken::ObjectEnd,
                        col: i,
                        line: self.current_line,
                    }),
                    '[' => self.tokens.push(Token {
                        value: JsonToken::ArrayStart,
                        col: i,
                        line: self.current_line,
                    }),
                    ']' => self.tokens.push(Token {
                        value: JsonToken::ArrayEnd,
                        col: i,
                        line: self.current_line,
                    }),
                    ':' => self.tokens.push(Token {
                        value: JsonToken::Colon,
                        col: i,
                        line: self.current_line,
                    }),
                    ',' => self.tokens.push(Token {
                        value: JsonToken::Comma,
                        col: i,
                        line: self.current_line,
                    }),
                    '0'..='9' => {
                        return NextStep::LexNumberType;
                    },
                    't' | 'f' => {
                        return NextStep::LexBoolean;
                    },
                    '"' => {
                        let last_added = &self.tokens.last().unwrap().value;

                        if last_added == &JsonToken::Comma || last_added == &JsonToken::ObjectStart {
                            return NextStep::LexName;
                        } else if last_added == &JsonToken::Colon {
                            return NextStep::LexString;
                        }
                    }
                    _ => ()
                }
            }
        }

        if let Some((i, line)) = self.lines.next() {
            self.char_iter = Some(line.chars().enumerate().peekable());
            self.current_line = i;
            return NextStep::LexCharacter
        }

        return NextStep::Done
    }

    fn lex<F: FnMut((&usize, &char)) -> NextLexStep>(&mut self, mut f: F) -> Option<usize>{
        let mut token_start = None;

        if let Some(char_iter) = &mut self.char_iter {
            while let Some((i, next_char)) = char_iter.peek() {
                if token_start.is_none() {
                    token_start = Some(i.clone());
                }

                match f((i, next_char)) {
                    NextLexStep::Advance => {
                        char_iter.next();
                    },
                    NextLexStep::Skip => {
                        char_iter.next();
                        char_iter.next();
                    },
                    NextLexStep::Done => break,
                }
            }
        }

        token_start
    }

    fn lex_boolean(&mut self) {
        let token_start = self.lex(|(i, next_char)| {
           match next_char {
               ',' => NextLexStep::Done,
               _ => NextLexStep::Advance,
           }
        });

        if let Some(token_start) = token_start {
            self.tokens.push(
                Token {
                    value: JsonToken::Value(JsonType::Bool),
                    col: token_start,
                    line: self.current_line,
                }
            )
        }
    }

    fn lex_name(&mut self) {
        let mut end_index = 0;
        let token_start = self.lex( |(i, next_char)| {
           match next_char {
               '\\' => NextLexStep::Advance,
               '"' => NextLexStep::Done,
               _ => {
                   end_index = *i;
                   return NextLexStep::Advance;
               }
           }
        });

        if let Some(token_start) = token_start {
            self.tokens.push(
                Token {
                    value: JsonToken::Name(&self.json[token_start..end_index + 1]),
                    col: token_start,
                    line: self.current_line
                }
            )
        }
    }

    fn lex_string(&mut self) {
        let token_start = self.lex(|(i, next_char)| {
            match next_char {
                '\\' => NextLexStep::Skip,
                '"' => NextLexStep::Done,
                _ => NextLexStep::Advance,
            }
        });

        if let Some(token_start) = token_start {
            self.tokens.push(
                Token{
                    value: JsonToken::Value(JsonType::String),
                    line: self.current_line,
                    col: token_start
                }
            );
        }
    }

    fn lex_number(&mut self) {
        let mut is_float = false;

        let token_start = self.lex(|(i, next_char)| {
            match next_char {
                '0'..='9' => NextLexStep::Advance,
                '.' => {
                    is_float = true;
                    return NextLexStep::Advance;
                },
                _ => NextLexStep::Done,
            }
        });

        if let Some(token_start) = token_start {
            self.tokens.push(
                Token {
                    value: JsonToken::Value(if is_float { JsonType::Float } else { JsonType::Int }),
                    col: token_start,
                    line: self.current_line,
                }
            );
        }
    }

    fn start_lex(mut self) -> Vec<Token<'a>> {
        let mut step = self.lex_character();
        while step != NextStep::Done {
            match step {
                NextStep::LexCharacter => step = self.lex_character(),
                NextStep::LexNumberType => {
                    step = LexCharacter;
                    self.lex_number();
                },
                NextStep::LexName => {
                    step = LexCharacter;
                    self.lex_name();
                },
                NextStep::LexString => {
                    step = LexCharacter;
                    self.lex_string();
                },
                NextStep::LexBoolean => {
                    step = LexCharacter;
                    self.lex_boolean();
                }
                _ => (),
            }
        }

        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::{Lexer, NextStep};
    use crate::parser::token::{JsonToken, JsonType};

    #[test]
    fn simple_json() {
        let json = "{\"f1\": \"value\", \"f2\": true, \"f3\": 45.3, \"f4\": 12}";

        let expected_result = vec![
            JsonToken::ObjectStart, JsonToken::Name("f1"), JsonToken::Colon, JsonToken::Value(JsonType::String),
            JsonToken::Comma, JsonToken::Name("f2"), JsonToken::Colon, JsonToken::Value(JsonType::Bool),
            JsonToken::Comma, JsonToken::Name("f3"), JsonToken::Colon, JsonToken::Value(JsonType::Float),
            JsonToken::Comma, JsonToken::Name("f4"), JsonToken::Colon, JsonToken::Value(JsonType::Int),
            JsonToken::ObjectEnd,
        ];

        let lexer = Lexer::new(json);

        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();
        assert_eq!(tokens, expected_result);
    }

    #[test]
    fn array_object_json() {
        let json = "{\"f1\": {\"f2\": true, \"f3\": 45.3, \"f4\": 12}, \"f2\": [1, 2, 3]}";
        let expected_result = vec![
            JsonToken::ObjectStart, JsonToken::Name("f1"), JsonToken::Colon, JsonToken::ObjectStart,
            JsonToken::Name("f2"), JsonToken::Colon, JsonToken::Value(JsonType::Bool), JsonToken::Comma,
            JsonToken::Name("f3"), JsonToken::Colon, JsonToken::Value(JsonType::Float), JsonToken::Comma,
            JsonToken::Name("f4"), JsonToken::Colon, JsonToken::Value(JsonType::Int), JsonToken::ObjectEnd,
            JsonToken::Comma, JsonToken::Name("f2"), JsonToken::Colon, JsonToken::ArrayStart,
            JsonToken::Value(JsonType::Int), JsonToken::Comma, JsonToken::Value(JsonType::Int), JsonToken::Comma,
            JsonToken::Value(JsonType::Int), JsonToken::ArrayEnd, JsonToken::ObjectEnd
        ];

        let lexer = Lexer::new(json);

        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();
        assert_eq!(tokens, expected_result);
    }

    #[test]
    fn lex_number() {
        let json = "5423234";
        let expected_result = vec![JsonToken::Value(JsonType::Int)];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();

        assert_eq!(expected_result, tokens);
    }

    #[test]
    fn lex_float() {
        let json = "542.3234";
        let expected_result = vec![JsonToken::Value(JsonType::Float)];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();

        assert_eq!(expected_result, tokens);
    }

    #[test]
    fn skip_number() {
        let json = "5423234,{";

        let mut lexer = Lexer::new(json);
        lexer.char_iter = Some(lexer.lines.next().unwrap().1.chars().enumerate().peekable());
        lexer.lex_number();
        let char = lexer.char_iter.unwrap().next().unwrap().1;

        assert_eq!(char, ',');
    }

    #[test]
    fn skip_float_number() {
        let json = "542.3234,{";

        let mut lexer = Lexer::new(json);
        lexer.char_iter = Some(lexer.lines.next().unwrap().1.chars().enumerate().peekable());
        lexer.lex_number();
        let char = lexer.char_iter.unwrap().next().unwrap().1;

        assert_eq!(char, ',');
    }

    #[test]
    fn lex_field_name() {
        let json = ",\"hola\"";
        let expected_result = vec![
            JsonToken::Comma,
            JsonToken::Name("hola")
        ];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();

        assert_eq!(tokens, expected_result);
    }

    #[test]
    fn lex_string() {
        let json = ":\"hola\"";
        let expected_result = vec![
            JsonToken::Colon,
            JsonToken::Value(JsonType::String),
        ];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();

        assert_eq!(tokens, expected_result);
    }

    #[test]
    fn lex_bool() {
        let json = "true";

        let expected_result = vec![
            JsonToken::Value(JsonType::Bool),
        ];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();

        assert_eq!(tokens, expected_result);
    }
}
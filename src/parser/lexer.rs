use std::iter::{Enumerate, Peekable};
use std::str::{Chars, Lines};
use crate::parser::token::{JsonToken, JsonType, Token};

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

    fn lex_character(&mut self) {
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
                        self.tokens.push(
                            Token {
                                value: JsonToken::Value(JsonType::Int),
                                col: i,
                                line: self.current_line,
                            }
                        );
                        return self.skip_number()
                    },
                    _ => ()
                }
            }
        }

        if let Some((i, line)) = self.lines.next() {
            self.char_iter = Some(line.chars().enumerate().peekable());
            self.current_line = i;
            return self.lex_character();
        }
    }

    fn skip_number(&mut self) {
        if let Some(char_iter) = &mut self.char_iter {
            if let Some((_, next_char)) = char_iter.peek() {
                return match next_char {
                    '0'..='9' => {
                        char_iter.next();
                        self.skip_number()
                    },
                    _ => {
                        return self.lex_character()
                    }
                }
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::Lexer;
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

        let tokens: Vec<JsonToken> = lexer.tokens.into_iter().map(|token| token.value).collect();
        assert_eq!(tokens, expected_result);
    }

    #[test]
    fn lex_number() {
        let json = "5423234";
        let expected_result = vec![JsonToken::Value(JsonType::Int)];

        let mut lexer = Lexer::new(json);
        lexer.lex_character();
        let tokens: Vec<JsonToken> = lexer.tokens.into_iter().map(|token| token.value).collect();

        assert_eq!(expected_result, tokens);
    }

    #[test]
    fn skip_number() {
        let json = "5423234,{";

        let mut lexer = Lexer::new(json);
        lexer.char_iter = Some(lexer.lines.next().unwrap().1.chars().enumerate().peekable());
        lexer.skip_number();

        let char = lexer.char_iter.unwrap().next().unwrap().1;

        assert_eq!(char, ',');
    }
}
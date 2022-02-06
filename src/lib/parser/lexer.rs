use std::iter::{Enumerate, Peekable};
use std::str::{Chars, Lines};
use crate::lib::parser::lexer::NextStep::{LexCharacter};
use crate::lib::parser::token::{JsonToken, JsonType, Token};


/// Next step for the character lexer.
#[derive(Debug, PartialEq, Eq)]
enum NextStep {
    LexNumberType,
    LexCharacter,
    LexName,
    LexString,
    LexBoolean,
    Done
}


/// Next Step for the lexer closure.
#[derive(Debug, PartialEq, Eq)]
enum NextLexStep {
    Done,
    Advance,
    Skip
}

pub struct Lexer<'a> {
    json: &'a str,
    lines: Enumerate<Lines<'a>>,
    current_line: usize,
    current_line_str: Option<&'a str>,
    char_iter: Option<Peekable<Enumerate<Chars<'a>>>>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {

    ///Creates a new lexer.
    /// # Parameters
    /// * `json` JSON String
    pub fn new(json: &'a str) -> Self {
        let lines = json.lines().enumerate();
        Self {
            json,
            lines,
            current_line: 0,
            current_line_str: None,
            char_iter: None,
            tokens: vec![],
        }
    }

    /// Processes basic tokens. Delegates to other functions for primitive types.
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
                        if let Some(last_token) = &self.tokens.last() {
                            let last_added = &last_token.value;
                            if last_added == &JsonToken::Comma || last_added == &JsonToken::ObjectStart {
                                return NextStep::LexName;
                            } else if last_added == &JsonToken::Colon {
                                return NextStep::LexString;
                            }
                        };
                    }
                    _ => ()
                }
            }
        }

        if let Some((i, line)) = self.lines.next() {
            self.current_line_str = Some(line);
            self.char_iter = Some(line.chars().enumerate().peekable());
            self.current_line = i;
            return NextStep::LexCharacter
        }

        return NextStep::Done
    }

    /// Basic lexer for primitive types. Runs a closure which returns the next step for the lexer (advance the iterator, skip a character or end the lexer).
    /// # Parameter
    /// * `f` - Closure which runs for each next characters. The iterator will be advanced (or not) depending of the returned value.
    /// # Returns
    /// Column of the first character of the token. For error message support.
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

    /// Processes a boolean datatype.
    fn lex_boolean(&mut self) {
        let token_start = self.lex(|(i, next_char)| {
            match next_char {
                ',' | '}' => NextLexStep::Done,
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

    /// Processes a field name.
    fn lex_name(&mut self) {
        let mut end_index = 0;


        let mut start_index = 0;
        let mut end_index = 0;

        let mut start_char = None;
        let mut end_char = None;

        if let Some(char_iter) = &mut self.char_iter {
            while let Some((i, char)) = char_iter.next() {
                if let Some((_, next_char)) = char_iter.peek() {
                    if start_char == None {
                        start_char = Some(char);
                        start_index = i;
                    }

                    if next_char == &'"' {
                        end_char = Some(char);
                        end_index = i;

                        break
                    }
                }
            }
        }


        if let Some(start_char) = start_char {
            if let Some(end_char) = end_char {
                if let Some(current_line_str) = self.current_line_str {
                    while current_line_str[start_index..=start_index] != start_char.to_string() {
                        start_index += 1;
                    }

                    while current_line_str[end_index..=end_index] != end_char.to_string() {
                        end_index += 1;
                    }

                    self.tokens.push(
                        Token {
                            value: JsonToken::Name(&current_line_str[start_index..=end_index]),
                            col: start_index,
                            line: self.current_line
                        }
                    )
                }
            }
        }
    }

    /// Processes a String value.
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

    /// Processes a number value. Defaults to adding a int token, will add a float token if it encounters a point(`.`) character.
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

    /// Consumes the structure and start the lexing process.
    /// # Returns
    /// Vec of Token structures.
    pub fn start_lex(mut self) -> Vec<Token<'a>> {
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


    #[test]
    fn lex_bad_name_after_degree_symbol() {
        let json = "{\"2\":\"aº\", \"ab\": 32}";

        let expected_result = vec![
            JsonToken::ObjectStart, JsonToken::Name("2"), JsonToken::Colon,
            JsonToken::Value(JsonType::String), JsonToken::Comma, JsonToken::Name("ab"),
            JsonToken::Colon, JsonToken::Value(JsonType::Int), JsonToken::ObjectEnd
        ];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();
        assert_eq!(tokens, expected_result)
    }

    #[test]
    fn lex_bool_end_on_right_brace() {
        let json =  "true}";
        let expected_result = vec![
            JsonToken::Value(JsonType::Bool), JsonToken::ObjectEnd
        ];

        let lexer = Lexer::new(json);
        let tokens: Vec<JsonToken> = lexer.start_lex().into_iter().map(|token| token.value).collect();
        assert_eq!(tokens, expected_result)
    }
}

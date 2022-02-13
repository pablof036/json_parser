use std::fs;
use crate::lib::model::transform_config::{DART_DEFINITION, JAVA_DEFINITION, RUST_DEFINITION};
use crate::lib::parser::lexer::Lexer;
use crate::lib::parser::tokenizer::Tokenizer;
use crate::lib::transformer::Transformer;

mod parser;
mod model;
mod transformer;
mod case;

pub fn run(filename: String) -> anyhow::Result<()> {
    let file = fs::read_to_string(filename)?;


    let lexer = Lexer::new(&file);
    let lexer_result = lexer.start_lex();
    let token = Tokenizer::new(lexer_result);
    let tokenizer_result = token.start_tokenizer()?;
    let transformer = Transformer::new(DART_DEFINITION, tokenizer_result, None)?;
    let result = transformer.start_transform();

    result.iter().rev().for_each(|object| object.iter().for_each(|string| {
       println!("{}", string)
    }));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lib::run;

    #[test]
    fn runs_on_valid_path() -> anyhow::Result<()>{
        run(String::from("test.json"))
    }

    #[test]
    #[should_panic]
    fn fails_on_valid_path() {
        run(String::from("a")).unwrap()
    }
}
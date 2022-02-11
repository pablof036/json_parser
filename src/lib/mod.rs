use std::fs;
use crate::lib::model::transform_config::RUST_DEFINITION;
use crate::lib::parser::lexer::Lexer;
use crate::lib::parser::tokenizer::Tokenizer;
use crate::lib::transformer::Transformer;

mod parser;
mod model;
mod transformer;

pub fn run(filename: String) -> anyhow::Result<()> {
    let file = fs::read_to_string(filename)?;

    let lexer = Lexer::new(&file);
    let lexer_result = lexer.start_lex();
    let token = Tokenizer::new(lexer_result);
    let tokenizer_result = token.start_tokenizer()?;
    let transformer = Transformer::new(RUST_DEFINITION, tokenizer_result, None)?;
    let result = transformer.start_transform();

    println!("{:#?}", result);

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
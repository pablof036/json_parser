use std::env::Args;
use std::{fs, process};
use std::path::Path;
use anyhow::bail;
use crate::HELP_MESSAGE;
use crate::lib::model::transform_config::{DART_DEFINITION, JAVA_DEFINITION, KOTLIN_DEFINITION, RUST_DEFINITION, TransformConfig};
use crate::lib::parser::lexer::Lexer;
use crate::lib::parser::tokenizer::Tokenizer;
use crate::lib::transformer::Transformer;

mod parser;
mod model;
mod transformer;
mod case;


pub struct Config {
    filename: String,
    transformer_config: TransformConfig,
}


impl Config {
    pub fn new(mut args: Args) -> anyhow::Result<Self> {
        let mut help = None;

        let mut definition_arg = None;

        let mut filename = None;

        args.skip(1).for_each(|arg| {
            if arg.contains("--definition") {
                definition_arg = Some(arg)
            } else if arg == "--help" {
                help = Some(arg);
            } else {
                filename = Some(arg);
            }
        });

        if help.is_some() {
            println!("{}", HELP_MESSAGE);
            process::exit(0);
        }

        let transformer_config = match definition_arg {
            Some(definition) => {
                let definition = match definition.split('=').last() {
                    Some(definition) => definition,
                    None => bail!("syntax error in definition argument")
                };

                match definition.as_ref() {
                    "kotlin" => KOTLIN_DEFINITION,
                    "rust" => RUST_DEFINITION,
                    "java" => JAVA_DEFINITION,
                    "dart" => DART_DEFINITION,
                    _ => {
                        if Path::new(definition).exists() {
                            Self::load_definition(definition)?
                        } else {
                            bail!("definition not found")
                        }
                    }
                }
            },
            None => bail!("definition not provided")
        };

        let filename = match filename {
            Some(filename) => filename,
            _ => bail!("filename not provided")
        };

        Ok(
            Config {
                filename,
                transformer_config
            }
        )
    }

    pub fn load_definition(path: &str) -> anyhow::Result<TransformConfig> {
        let definition_file = fs::read_to_string(path)?;
        let config: TransformConfig = toml::from_str(&definition_file)?;
        Ok(config)
    }
}

pub fn run(config: Config) -> anyhow::Result<()> {
    let file = fs::read_to_string(config.filename)?;


    let lexer = Lexer::new(&file);
    let lexer_result = lexer.start_lex();
    let token = Tokenizer::new(lexer_result);
    let tokenizer_result = token.start_tokenizer()?;
    let transformer = Transformer::new(config.transformer_config, tokenizer_result, None)?;
    let result = transformer.start_transform();

    result.iter().rev().for_each(|object| object.iter().for_each(|string| {
       println!("{}", string)
    }));

    Ok(())
}

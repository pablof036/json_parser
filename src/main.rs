use std::{env, process};
use crate::lib::Config;

mod lib;

const HELP_MESSAGE: &'static str = r#"Usage: json-parser --definition="definition" file_name
Availabble definitions: rust, java, kotlin, dart.
You can also provide the path of a custom definition in a .toml file.
Because the type of a value needs to be inferred, neither null values nor empty arrays are supported."#;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{}\n{}", e, HELP_MESSAGE);
        process::exit(1);
    });

    if let Err(e) = lib::run(config) {
        eprintln!("Error while running: {}.\n{}", e, HELP_MESSAGE);
    }
}

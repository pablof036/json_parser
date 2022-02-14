use std::{env, process};
use crate::lib::Config;

mod lib;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    if let Err(e) = lib::run(config) {
        eprintln!("Error while running: {}", e);
    }
}

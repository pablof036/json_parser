use std::{env, process};
use crate::lib::run;

mod parser;
mod lib;

fn main() {
    let filename = env::args().skip(1).next().unwrap_or_else(|| {
        eprintln!("Not enough arguments provided! Need filename");
        process::exit(1);
    });

    if let Err(e) = run(filename) {
        eprintln!("Error while running: {}", e);
    }
}
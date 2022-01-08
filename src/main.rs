use crate::lib::{Config, Operation};
use std::env;
mod lib;

fn main() {
    match Config::new(env::args()) {
        Ok(config) => run(config),
        Err(message) => eprintln!("{}", message),
    }
}

fn run(config: Config) {
    match config.operation {
        Operation::SingleFile => lib::split_file(config.path()),
        Operation::Directory => run_directory(config),
    };
}

fn run_directory(config: Config) {
    eprintln!("Splitting all files in {:?}", &config.path());

    for path in config.paths() {
        lib::split_file(&path);
        eprintln!();
    }
}

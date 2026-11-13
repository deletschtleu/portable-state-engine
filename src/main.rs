use std::env;
use std::process;

use portable_state_engine::cli::run_cli;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match run_cli(&args) {
        Ok(output) => {
            print!("{}", output);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

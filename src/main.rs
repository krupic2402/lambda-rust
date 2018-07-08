extern crate lambda_rust;
extern crate rustyline;

use lambda_rust::runtime::*;
use rustyline::error::ReadlineError;
use std::process;

fn main() {
    let mut editor = rustyline::Editor::<()>::new();
    let mut runtime: Environment<HashSymbolTable> = Environment::new();

    loop {
        let input = match editor.readline("> ") {
            Ok(line) => {
                editor.add_history_entry(&line);
                line
            }
            Err(ReadlineError::Interrupted) => {
                exit();
            }
            e @ Err(_) => {
                println!("Error: {:?}", e);
                continue;
            }
        };

        let input = input.trim();
        if input.is_empty() { continue; }

        if input.starts_with(':') {
            let command = &input[1..];
            if command.is_empty() { continue; }

            if "quit".starts_with(command) {
                exit();
            }

            continue;
        }

        runtime.interpret(input);
    }
}

fn exit() -> ! {
    println!("Exiting ...");
    process::exit(0);
}

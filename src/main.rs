extern crate lambda_rust;
extern crate rustyline;
extern crate isatty;

use lambda_rust::runtime::*;
use rustyline::{error::ReadlineError, config::{Config, CompletionType}};
use isatty::*;
use std::process;

mod commands;
use commands::{Command, Commands};

fn main() {
    let commands = Commands::new()
                        .add(Command("quit"))
                        .add(Command("exit"))
                        .done();

    let mut editor = rustyline::Editor::<&Commands>::with_config(
        Config::builder().completion_type(CompletionType::List).build());
    editor.set_completer(Some(&commands));

    let mut runtime: Environment<HashSymbolTable> = Environment::new();

    loop {
        let input = match editor.readline("> ") {
            Ok(line) => {
                editor.add_history_entry(line.clone());
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                exit();
            }
            e @ Err(_) => {
                println!("Error: {:?}", e);
                continue;
            }
        };

        let input = input.trim();
        if input.is_empty() { continue; }

        if input.starts_with(commands::COMMAND_PREFIX) {
            match commands.parse(input) {
                Err(e) => println!("{}", e),
                Ok(c) => match c.command {
                    "quit" | "exit" => exit(),
                    _ => unreachable!(),
                }
            }

            continue;
        }

        let _ = runtime.interpret(input);
    }
}

fn exit() -> ! {
    if stdin_isatty() {
        println!("Exiting ...");
    }
    process::exit(0);
}

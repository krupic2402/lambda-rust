extern crate lambda_rust;
extern crate rustyline;
extern crate isatty;
#[macro_use] extern crate lazy_static;

use lambda_rust::runtime::*;
use rustyline::{error::ReadlineError, config::{Config, CompletionType}};
use isatty::*;
use std::process;
use std::fs::File;
use std::io::{BufReader, BufRead};

mod commands;
use commands::{Command, Commands, CommandCall, ArgType, completion::Completers};

const QUIT: &str = "quit";
const EXIT: &str = "exit";
const SHOW: &str = "show";
const LIST: &str = "list";
const IMPORT: &str = "import";
const ECHO: &str = "echo";
const REDUCTIONS: &str = "reductions";

fn main() {
    let commands = Commands::new()
                        .with_completers(Completers::default())
                        .add(Command::nullary(QUIT))
                        .add(Command::nullary(EXIT))
                        .add(Command::new(SHOW, ArgType::Symbol))
                        .add(Command::nullary(LIST))
                        .add(Command::with_arity(IMPORT, ArgType::File, 1))
                        .add(Command::with_arity(ECHO, ArgType::Boolean, 1))
                        .add(Command::with_arity(REDUCTIONS, ArgType::Number, 1))
                        .done();

    let mut editor = rustyline::Editor::<&Commands<Completers<_>>>::with_config(
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
                Ok(c) => match c.command.name {
                    QUIT | EXIT => exit(),
                    SHOW => show(c, &runtime),
                    LIST => list(&runtime),
                    IMPORT => import(c, &mut runtime),
                    ECHO => set_echo(c, &mut runtime),
                    REDUCTIONS => set_max_reductions(c, &mut runtime),
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

fn set_echo(command: CommandCall, runtime: &mut Environment) {
    match command.args[0].parse() {
        Ok(b) => runtime.echo_enabled = b,
        Err(e) => println!("Error: {}", e),
    }
}

fn set_max_reductions(command: CommandCall, runtime: &mut Environment) {
    match command.args[0].parse() {
        Ok(u) => runtime.max_reductions = u,
        Err(e) => println!("Error: {}", e),
    }
}

fn show(command: CommandCall, runtime: &Environment) {
    for identifier in command.args {
        match runtime.symbol_table().get(identifier) {
            Some(term) => println!("{} = {}", identifier, term),
            None => println!("Undefined identifier \"{}\"", identifier),
        }
    }
}

fn list(runtime: &Environment) {
    let mut bindings: Vec<_> = runtime.symbol_table().bindings().collect();
    bindings.sort_unstable_by_key(|b| b.0);
    for (name, term) in bindings {
        println!("{} = {}", name, term);
    }
}

fn import(command: CommandCall, runtime: &mut Environment) {
    let filename = command.args[0];
    match File::open(filename) {
        Err(e) => println!("Error opening {}: {}", filename, e),
        Ok(file) => {
            let mut reader = BufReader::new(&file);
            for (line_number, line) in reader.lines().enumerate() {
                match runtime.interpret(&line.unwrap()) {
                    Err(_) => {
                        println!("Error in line {}.", line_number + 1);
                        break;
                    }
                    Ok(_) => continue,
                }
            }
        }
    }
} 

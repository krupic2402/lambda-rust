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
use std::sync::{Arc, Mutex};

mod commands;
use commands::{Command, Commands, CommandCall, ArgType};

mod completion;
use completion::{Completers, completers::SymbolTableAdapter};

const QUIT: &str = "quit";
const EXIT: &str = "exit";
const SHOW: &str = "show";
const LIST: &str = "list";
const IMPORT: &str = "import";
const ECHO: &str = "echo";
const REDUCTIONS: &str = "reductions";

fn main() {
    let runtime: Arc<Mutex<Environment<HashSymbolTable>>> = Arc::new(Mutex::new(Environment::new()));

    let completers = Completers::default().add(ArgType::Symbol, Box::new(SymbolTableAdapter::new(&runtime)));

    let commands = Commands::new()
                        .with_completers(completers)
                        .with_help()
                        .add(Command::nullary(QUIT))
                        .add(Command::nullary(EXIT))
                        .add(Command::new(SHOW, ArgType::Symbol))
                        .add(Command::nullary(LIST))
                        .add(Command::unary(IMPORT, ArgType::File))
                        .add(Command::with_arities(ECHO, ArgType::Boolean, vec![0, 1]))
                        .add(Command::with_arities(REDUCTIONS, ArgType::Number, vec![0, 1]))
                        .done();

    let mut editor = rustyline::Editor::<&Commands<Completers<_>>>::with_config(
        Config::builder().completion_type(CompletionType::List).build());
    editor.set_completer(Some(&commands));


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

        let mut runtime_lock = runtime.lock().unwrap();

        let input = input.trim();
        if input.is_empty() { continue; }

        if input.starts_with(commands::COMMAND_PREFIX) {
            match commands.parse(input) {
                Err(e) => println!("{}", e),
                Ok(c) => match c.command.name {
                    QUIT | EXIT => exit(),
                    SHOW => show(c, &runtime_lock),
                    LIST => list(&runtime_lock),
                    IMPORT => import(c, &mut runtime_lock),
                    ECHO => set_or_print_echo(c, &mut runtime_lock),
                    REDUCTIONS => set_or_print_max_reductions(c, &mut runtime_lock),
                    commands::HELP_COMMAND => {
                        let format = format::Fmt(|mut f| {
                            commands.write_help(&mut f, c.args.get(0).map(|a| *a))
                        });
                        println!("{}", format);
                    }
                    _ => unreachable!(),
                }
            }

            continue;
        }

        let _ = runtime_lock.interpret(input);
    }
}

fn exit() -> ! {
    if stdin_isatty() {
        println!("Exiting ...");
    }
    process::exit(0);
}

fn set_or_print_echo(command: CommandCall, runtime: &mut Environment) {
    match command.args.as_slice() {
        [] => println!("Echo: {}", runtime.echo_enabled),
        [boolean] => match boolean.parse() {
            Ok(b) => runtime.echo_enabled = b,
            Err(e) => println!("Error: {}", e),
        }
        _ => unreachable!(),
    }
}

fn set_or_print_max_reductions(command: CommandCall, runtime: &mut Environment) {
    match command.args.as_slice() {
        [] => println!("Maximum reductions: {}", runtime.max_reductions),
        [number] => match number.parse() {
            Ok(u) => runtime.max_reductions = u,
            Err(e) => println!("Error: {}", e),
        }
        _ => unreachable!(),
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

mod format {
    use std::fmt::{Formatter, Result, Display};
    pub struct Fmt<F>(pub F) where F: Fn(&mut Formatter) -> Result;

    impl<F> Display for Fmt<F> where F: Fn(&mut Formatter) -> Result {
        fn fmt(&self, f: &mut Formatter) -> Result {
            (self.0)(f)
        } 
    } 
}

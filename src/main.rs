extern crate lambda_rust;
extern crate rustyline;

use lambda_rust::{
    lexer::Token,
    parser::parse,
    runtime::*,
};

use rustyline::error::ReadlineError;

use std::process;

fn main() {
    let mut editor = rustyline::Editor::<()>::new();
    let mut runtime = Environment::new();

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

        interpret(input, &mut runtime);
    }
}

fn interpret<S: AsRef<str>>(input: S, env: &mut impl SymbolTable) {
    let tokens = Token::parse_all(input.as_ref());
    if let Err(ref e) = tokens {
        println!("{}", e.0);
        return;
    }

    let tokens = tokens.unwrap();
    let statement = parse(&tokens);
    match statement {
        Err(ref e) => {
            println!("{}", e);
            return;
        }
        Ok(Statement::LetStatement(binding)) => {
            add_binding(binding, env);
        }
        Ok(Statement::Expression(term)) => {
            println!(" : {}", term);
            evaluate(term, env);
        }
    }
}

fn exit() -> ! {
    println!("Exiting ...");
    process::exit(0);
}

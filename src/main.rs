extern crate lambda_rust;
extern crate rustyline;

use lambda_rust::{
    lambda::*,
    lexer::Token,
    parser::parse,
    runtime::*,
};

use rustyline::error::ReadlineError;

use std::collections::HashSet;
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

const MAX_REDUCTIONS: usize = 5000;

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

fn add_binding(binding: Binding, symbols: &mut impl SymbolTable) {
    let mut binding = binding.map_term(|t| t.bind_free_from(symbols));
    // if, after binding predefined values, the term still references
    // the name it is being bound to, reject
    if binding.value.is_free_in(&binding.identifier) {
        println!("Error: recursive binding");
        return;
    }

    match binding.mode {
        BindMode::CaptureOnly => symbols.insert(binding),
        BindMode::CaptureAndReduce => {
            match evaluate(binding.value, &()) {
                Some(term) => {
                    binding.value = term;
                    symbols.insert(binding);
                }
                None => println!("Error reducing term"),
            }
        }
    }

}

fn evaluate(mut term: Term, symbols: &impl SymbolTable) -> Option<Term> {
    term = term.bind_free_from(symbols);
 
    let mut seen_terms = HashSet::new();
    loop {
        if seen_terms.len() > MAX_REDUCTIONS {
            println!("[too many reductions]");
            return None;
        }

        let reduct = term.reduce(Strategy::NormalOrder);
        print!("β: ");
        match reduct {
            EvalResult::NormalForm(r) => {
                println!("{} [normal]", r);
                return Some(r);
            }
            EvalResult::PossiblyReducible(r) => {
                if !seen_terms.contains(&r) {
                    println!("{}", r);
                    term = r;
                    seen_terms.insert(term.clone());
                } else {
                    println!("[non-terminating]");
                    return None;
                }
            }
        }
    }
}

fn exit() -> ! {
    println!("Exiting ...");
    process::exit(0);
}

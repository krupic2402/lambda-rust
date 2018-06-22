extern crate lambda_rust;

use lambda_rust::{
    lambda::*,
    lexer::Token,
    parser::parse,
};

use std::io::{self, Write};
use std::collections::HashSet;

fn main() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout.");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input.");

        if ":quit".starts_with(input.trim()) && input.trim().len() > 1 {
            break;
        }

        let tokens = Token::parse_all(&input);
        if let Err(ref e) = tokens {
            println!("{:?}", e);
            continue;
        }

        let tokens = tokens.unwrap();
        let term = parse(&tokens);
        match term {
            Err(ref e) => {
                println!("{:?}", e);
                continue;
            }
            Ok(ref t) => println!(" : {}", t),
        }


        let mut term = term.unwrap();
        let mut seen_terms = HashSet::new();
        loop {
            let reduct = term.reduce(Strategy::NormalOrder);
            print!("β: ");
            match reduct {
                EvalResult::NormalForm(r) => {
                    println!("{} [normal]", r);
                    break;
                }
                EvalResult::PossiblyReducible(r) => {
                    if !seen_terms.contains(&r) {
                        println!("{}", r);
                        term = r;
                        seen_terms.insert(term.clone());
                    } else {
                        println!("[non-terminating]");
                        break;
                    }
                }
            }
        }
    }
}

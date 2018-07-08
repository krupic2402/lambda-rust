use ::lambda::{self, Term, Strategy};
use ::lexer::Token;
use ::parser::parse;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BindMode {
    CaptureAndReduce,
    CaptureOnly,
}

#[derive(Debug, PartialEq)]
pub struct Binding {
    identifier: String,
    value: Term,
    mode: BindMode,
}

impl Binding {
    pub fn new<S: Into<String>>(identifier: S, value: Term, mode: BindMode) -> Binding {
        Binding {
            identifier: identifier.into(),
            value,
            mode,
        }
    }
}

pub trait SymbolTable {
    fn insert(&mut self, binding: Binding);
    fn get(&self, identifier: &String) -> Option<&Term>;
}

pub type HashSymbolTable = HashMap<String, Term>;

impl SymbolTable for HashSymbolTable {
    fn insert(&mut self, binding: Binding) {
        self.insert(binding.identifier, binding.value);
    }

    fn get(&self, identifier: &String) -> Option<&Term> {
        self.get(identifier)
    }
}

impl SymbolTable for () {
    #[allow(unused_variables)]
    fn insert(&mut self, binding: Binding) {}

    #[allow(unused_variables)]
    fn get(&self, identifier: &String) -> Option<&Term> {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EvaluationError {
    TooManyReductions,
    NonTerminating,
    RecursiveBinding,
}

use self::EvaluationError::*;

pub type EvaluationResult<T> = Result<T, EvaluationError>;

pub struct Environment<T: SymbolTable = HashSymbolTable> {
    symbols: T,
    max_reductions: usize,
}

impl<T: SymbolTable> Environment<T> {
    const MAX_REDUCTIONS_DEFAULT: usize = 5000;
    const ANS: &'static str = "ans";

    pub fn new() -> Environment<T> where T: Default {
        Environment {
            symbols: T::default(),
            max_reductions: Self::MAX_REDUCTIONS_DEFAULT,
        }
    }

    fn add_binding(&mut self, mut binding: Binding) -> EvaluationResult<()> {
        // always capture free variables from environment
        binding.value = binding.value.bind_free_from(&self.symbols);

        // if, after binding predefined values, the term still references
        // the name it is being bound to, reject
        if binding.value.is_free_in(&binding.identifier) {
            println!("Error: recursive binding");
            return Err(RecursiveBinding);
        }

        if let BindMode::CaptureAndReduce = binding.mode {
            binding.value = self.evaluate(binding.value)?;
        }

        self.symbols.insert(binding);
        return Ok(());
    }

    fn evaluate(&self, mut term: Term) -> EvaluationResult<Term> {
        term = term.bind_free_from(&self.symbols);

        let mut seen_terms = HashSet::new();
        loop {
            if seen_terms.len() > self.max_reductions {
                println!("[too many reductions]");
                return Err(TooManyReductions);
            }

            let reduct = term.reduce(Strategy::NormalOrder);
            print!("β: ");
            match reduct {
                lambda::EvalResult::NormalForm(r) => {
                    println!("{} [normal]", r);
                    return Ok(r);
                }
                lambda::EvalResult::PossiblyReducible(r) => {
                    if !seen_terms.contains(&r) {
                        println!("{}", r);
                        term = r;
                        seen_terms.insert(term.clone());
                    } else {
                        println!("[non-terminating]");
                        return Err(NonTerminating);
                    }
                }
            }
        }
    }

    pub fn interpret<S: AsRef<str>>(&mut self, input: S) -> EvaluationResult<()> {
        let tokens = Token::parse_all(input.as_ref());
        if let Err(ref e) = tokens {
            println!("{}", e.0);
            return Ok(());
        }

        let tokens = tokens.unwrap();
        let statement = parse(&tokens);
        match statement {
            Err(ref e) => {
                println!("{}", e);
                return Ok(());
            }
            Ok(Statement::LetStatement(binding)) => {
                self.add_binding(binding)?;
            }
            Ok(Statement::Expression(term)) => {
                println!(" : {}", term);
                let ans = Binding::new(Self::ANS, term, BindMode::CaptureAndReduce);
                self.add_binding(ans)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Binding),
    Expression(Term),
}

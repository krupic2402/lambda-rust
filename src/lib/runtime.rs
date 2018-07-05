use ::lambda::{self, Term, Strategy};

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

pub type Environment = HashMap<String, Term>;

impl SymbolTable for Environment {
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

pub fn add_binding(mut binding: Binding, symbols: &mut impl SymbolTable) -> EvaluationResult<()> {
    binding.value = binding.value.bind_free_from(symbols);

    // if, after binding predefined values, the term still references
    // the name it is being bound to, reject
    if binding.value.is_free_in(&binding.identifier) {
        println!("Error: recursive binding");
        return Err(RecursiveBinding);
    }

    if let BindMode::CaptureAndReduce = binding.mode {
        binding.value = evaluate(binding.value, &())?;
    }

    symbols.insert(binding);
    return Ok(());
}

const MAX_REDUCTIONS: usize = 5000;

pub fn evaluate(mut term: Term, symbols: &impl SymbolTable) -> EvaluationResult<Term> {
    term = term.bind_free_from(symbols);

    let mut seen_terms = HashSet::new();
    loop {
        if seen_terms.len() > MAX_REDUCTIONS {
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

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Binding),
    Expression(Term),
}

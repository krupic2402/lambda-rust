use ::lambda::{self, Term, Strategy};
use ::lexer::Token;
use ::parser::parse;
use std::collections::{HashMap, HashSet};
use std::iter;

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
    fn get(&self, identifier: &str) -> Option<&Term>;
    fn symbols<'a>(&'a self) -> Box<dyn Iterator<Item = &'a String> + 'a>;
    fn bindings<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Term)> + 'a>;
}

pub type HashSymbolTable = HashMap<String, Term>;

impl SymbolTable for HashSymbolTable {
    fn insert(&mut self, binding: Binding) {
        self.insert(binding.identifier, binding.value);
    }

    fn get(&self, identifier: &str) -> Option<&Term> {
        self.get(identifier)
    }

    fn symbols<'a>(&'a self) -> Box<dyn Iterator<Item = &'a String> + 'a> {
        Box::new(self.keys())
    }

    fn bindings<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a String, &'a Term)> + 'a> {
        Box::new(self.iter())
    }
}

impl SymbolTable for () {
    #[allow(unused_variables)]
    fn insert(&mut self, binding: Binding) {}

    #[allow(unused_variables)]
    fn get(&self, identifier: &str) -> Option<&Term> {
        None
    }

    fn symbols(&self) -> Box<dyn Iterator<Item = &String>> {
        Box::new(iter::empty())
    }

    fn bindings(&self) -> Box<dyn Iterator<Item = (&String, &Term)>> {
        Box::new(iter::empty())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EvaluationError {
    TooManyReductions,
    NonTerminating,
    RecursiveBinding,
    ParseError,
}

use self::EvaluationError::*;

pub type EvaluationResult<T> = Result<T, EvaluationError>;

pub struct Environment<T: SymbolTable = HashSymbolTable> {
    symbols: T,
    pub max_reductions: usize,
    pub echo_enabled: bool,
}

#[allow(unknown_lints,new_without_default)]
impl<T: SymbolTable> Environment<T> {
    const MAX_REDUCTIONS_DEFAULT: usize = 5000;
    const ANS: &'static str = "ans";

    pub fn new() -> Environment<T> where T: Default {
        Environment {
            symbols: T::default(),
            max_reductions: Self::MAX_REDUCTIONS_DEFAULT,
            echo_enabled: true,
        }
    }

    pub fn symbol_table(&self) -> &impl SymbolTable {
        &self.symbols
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
        Ok(())
    }

    fn evaluate(&self, mut term: Term) -> EvaluationResult<Term> {
        term = term.bind_free_from(&self.symbols);

        let mut seen_terms = HashSet::new();
        let mut reduction_count: usize = 0;
        loop {
            if reduction_count > self.max_reductions {
                println!("[too many reductions: {}]", reduction_count);
                return Err(TooManyReductions);
            }

            let reduct = term.reduce(Strategy::NormalOrder);
            match reduct {
                lambda::EvalResult::NormalForm(r) => {
                    println!("β: {} [normal; {} reductions]", r, reduction_count);
                    return Ok(r);
                }
                lambda::EvalResult::PossiblyReducible(r) => {
                    if !seen_terms.contains(&r) {
                        if self.echo_enabled { println!("β: {}", r); }
                        term = r;
                        seen_terms.insert(term.clone());
                        reduction_count += 1;
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
            return Err(ParseError);
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
                if self.echo_enabled { println!(" : {}", term); }
                let ans = Binding::new(Self::ANS, term, BindMode::CaptureAndReduce);
                self.add_binding(ans)?;
            }
        }

        Ok(())
    }
}

impl<T: SymbolTable + Default> Default for Environment<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Binding),
    Expression(Term),
}

use ::lambda::Term;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BindMode {
    CaptureAndReduce,
    CaptureOnly,
}

#[derive(Debug, PartialEq)]
pub struct Binding {
    pub identifier: String,
    pub value: Term,
    pub mode: BindMode,
}

impl Binding {
    pub fn new<S: Into<String>>(identifier: S, value: Term, mode: BindMode) -> Binding {
        Binding {
            identifier: identifier.into(),
            value,
            mode,
        }
    }

    pub fn map_term<F: FnOnce(Term) -> Term>(mut self, f: F) -> Binding  {
        self.value = f(self.value);
        self
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

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Binding),
    Expression(Term),
}

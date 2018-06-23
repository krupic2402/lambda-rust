use ::lambda::Term;

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Binding {
    identifier: String,
    value: Term,
}

impl Binding {
    pub fn new<S: Into<String>>(identifier: S, value: Term) -> Binding {
        Binding {
            identifier: identifier.into(),
            value,
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable(HashMap<String, Term>);

impl SymbolTable {
    pub fn insert(&mut self, binding: Binding) {
        self.0.insert(binding.identifier, binding.value);
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    LetStatement(Binding),
    Expression(Term),
}

use ::lambda::Term;

use std::collections::HashMap;

#[derive(Debug)]
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

pub struct SymbolTable(HashMap<String, Term>);

impl SymbolTable {
    pub fn insert(&mut self, binding: Binding) {
        self.0.insert(binding.identifier, binding.value);
    }
}

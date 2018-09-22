extern crate char_iter;

use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use rustyline::completion::Completer;

pub trait CompleterProvider<T: PartialEq> {
    fn get_completer(&self, token_type: &T) -> &dyn Completer;
}

pub struct Completers<T: Eq + Hash> {
    completers: HashMap<T, Box<dyn Completer>>,
}

impl<T: Eq + Hash> Completers<T> {
    pub fn new() -> Self {
        Completers { completers: HashMap::new() }
    }

    pub fn add(mut self, token_type: T, completer: Box<dyn Completer>) -> Self {
       self.completers.insert(token_type, completer);
       self
    }
}

impl<T: Eq + Hash> CompleterProvider<T> for Completers<T> {
    fn get_completer(&self, token_type: &T) -> &dyn Completer {
        self.completers.get(token_type).map(|b| &**b).unwrap_or(&() as &dyn Completer)
    }
}


lazy_static! {
    pub static ref WHITESPACE: BTreeSet<char>  = {
        let mut ws = BTreeSet::new();
        ws.extend("\u{0020}\u{0085}\u{00A0}\u{1680}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}".chars());
        ws.extend(char_iter::new('\u{0009}', '\u{000D}'));
        ws.extend(char_iter::new('\u{2000}', '\u{200A}'));
        ws
    };
}

pub mod completers {
    extern crate rustyline;
    extern crate lambda_rust;

    use rustyline::completion::{extract_word, Completer};
    use lambda_rust::runtime::{Environment, SymbolTable};
    use std::cmp::min;
    use std::sync::{Arc, Weak, Mutex};
    use super::WHITESPACE;

    pub struct BoolCompleter;

    impl Completer for BoolCompleter {
        fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
            let (mut word_start, word) = extract_word(line, pos, None, &WHITESPACE);
            let mut matches = vec![];
            if "true".starts_with(word) {
                matches.push("true".into());
            }
            if "false".starts_with(word) {
                matches.push("false".into());
            }
            if matches.is_empty() {
                word_start = 0;
            }
            Ok((word_start, matches))
        }
    }

    pub struct SymbolTableAdapter<T: SymbolTable>(Weak<Mutex<Environment<T>>>);

    impl<T: SymbolTable> SymbolTableAdapter<T> {
        pub fn new(environment: &Arc<Mutex<Environment<T>>>) -> Self {
            SymbolTableAdapter(Arc::downgrade(environment))
        }
    }

    impl<T: SymbolTable> Completer for SymbolTableAdapter<T> {
        fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
            match self.0.upgrade() {
                None  => ().complete(line, pos),
                Some(runtime) => {
                    let lock = runtime.lock().unwrap();
                    let symbols = lock.symbol_table().symbols();
                    let (word_start, word) = extract_word(line, pos, None, &WHITESPACE);
                    let prefix = &line[word_start..min(pos, word_start + word.len())];
                    let candidates: Vec<_> = symbols.filter(|s| s.starts_with(prefix)).cloned().collect();
                    Ok((word_start, candidates))
                }
            }
        }
    }
}

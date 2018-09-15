extern crate rustyline;

use rustyline::completion::Completer;
use std::str::SplitWhitespace;
use std::cmp::min;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ArgType {
    Symbol,
    File,
    Boolean,
    Number,
}

pub mod completion {
    use std::collections::HashMap;
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

    extern crate char_iter;
    use std::collections::BTreeSet;

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
        use rustyline::completion::{extract_word, Completer};
        pub struct BoolCompleter;

        impl Completer for BoolCompleter {
            fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
                let (mut word_start, word) = extract_word(line, pos, None, &super::WHITESPACE);
                let mut matches = vec![];
                if "true".starts_with(word) {
                    matches.push("true".into());
                }
                if "false".starts_with(word) {
                    matches.push("false".into());
                }
                if matches.len() == 0 {
                    word_start = 0;
                }
                Ok((word_start, matches))
            }
        }
    }
}

use self::completion::*;

impl Default for Completers<ArgType> {
    fn default() -> Self {
        Completers::new()
            .add(ArgType::Boolean, Box::new(completion::completers::BoolCompleter))
            .add(ArgType::File, Box::<rustyline::completion::FilenameCompleter>::default())
    }
}


#[derive(Debug, PartialEq)]
pub struct Command<'name> {
    pub name: &'name str,
    pub arity: Option<usize>,
    pub arg: Option<ArgType>,
}

impl<'name> Command<'name> {
    pub fn new(name: &str, arg: ArgType) -> Command {
        Command { name, arity: None, arg: Some(arg) }
    }

    pub fn with_arity(name: &str, arg: ArgType, arity: usize) -> Command {
        Command { name, arity: Some(arity), arg: Some(arg) }
    }

    pub fn nullary(name: &str) -> Command {
        Command { name, arity: Some(0), arg: None }
    }
}

pub struct Commands<'commands, T: CompleterProvider<ArgType>> {
    commands: Vec<Command<'commands>>,
    completers: Option<T>
}

pub struct Builder<'commands, T: CompleterProvider<ArgType>> {
    commands: Vec<Command<'commands>>,
    completers: Option<T>
}

impl<'commands, T: CompleterProvider<ArgType>> Builder<'commands, T> {
    pub fn add(mut self, command: Command<'commands>) -> Builder<'commands, T> {
        self.commands.push(command);
        self
    }

    pub fn with_completers(mut self, provider: T) -> Builder<'commands, T> {
        self.completers = Some(provider);
        self
    }

    pub fn done(self) -> Commands<'commands, T> {
        Commands { commands: self.commands, completers: self.completers }
    }
}

pub const COMMAND_PREFIX: &str = ":";

#[derive(Debug, PartialEq)]
pub struct InvalidCommand<'line>(&'line str);

impl<'line> Display for InvalidCommand<'line> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Invalid command: {}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandCall<'line, 'command> {
    pub command: &'command Command<'command>,
    pub args: Vec<&'line str>,
}

impl<'line, 'command> Display for CommandCall<'line, 'command> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", COMMAND_PREFIX, self.command.name)?;
        for arg in &self.args {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}

type ParseResult<'line, 'command> = Result<CommandCall<'line, 'command>, InvalidCommand<'line>>;

fn tokenize(line: &str) -> Option<(&str, usize, SplitWhitespace)> {
    let start = line.find(COMMAND_PREFIX);
    if start.is_none() {
        return None;
    }

    let start = start.unwrap() + 1;
    let mut tokens = line[start..].split_whitespace();
    let command = tokens.next();
    let command_prefix = command.unwrap_or("");
    let command_start = command.and_then(|c| line.find(c)).unwrap_or(start);

    Some((command_prefix, command_start, tokens))
}

impl<'commands, T: CompleterProvider<ArgType>> Commands<'commands, T> {

    pub fn new() -> Builder<'commands, T> {
        Builder { commands: vec![], completers: None }
    }

    pub fn match_str<'line>(&self, command: &'line str) -> Vec<&Command<'commands>> {
        self.commands.iter().filter(|c| c.name.starts_with(command)).collect()
    }

    pub fn parse<'line>(&'commands self, line: &'line str) -> ParseResult<'line, 'commands> {
        match tokenize(line) {
            Some((command, _, args)) => {
                let candidates = self.match_str(command);
                if candidates.len() == 1 {
                    let command = candidates[0];
                    let args = match command.arity {
                        Some(n) => {
                            let args: Vec<_> = args.take(n).collect();
                            if args.len() == n {
                                args
                            } else {
                                return Err(InvalidCommand(line));
                            }
                        }
                        None => args.collect(),
                    };

                    Ok(CommandCall { command, args })
                } else {
                    Err(InvalidCommand(line))
                }
            }
            _ => Err(InvalidCommand(line))
        }
    }
}

impl<'commands, T: CompleterProvider<ArgType>> Completer for Commands<'commands, T> {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let (full_word, position, _) = match tokenize(line) {
            Some(tuple) => tuple,
            None => return Ok((0, vec![])),
        };

        // need this condition because rustyline panics otherwise
        if pos < position {
            return Ok((0, vec![]));
        }

        let clamped_prefix = &line[position..min(pos, position + full_word.len())];
        let command_candidates = self.match_str(clamped_prefix);
        if command_candidates.len() == 1 {
            let command = command_candidates[0];

            if pos <= position + full_word.len() {
                Ok((position, vec![command.name.into()]))
            } else {
                let completer = command.arg.and_then(|at| self.completers.as_ref().map(|c| c.get_completer(&at))).unwrap_or(&());
                completer.complete(line, pos)
            }
        } else {
            let command_names = command_candidates.into_iter().map(|c| c.name.into()).collect();
            Ok((position, command_names))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matching() {
        let commands: Commands<Completers<_>> = Commands::new()
                                                .add(Command::nullary("abc"))
                                                .add(Command::nullary("def"))
                                                .add(Command::nullary("ddd"))
                                                .done();
        assert_eq!(vec![&Command::nullary("abc")], commands.match_str("a"));
        assert_eq!(vec![&Command::nullary("def"), &Command::nullary("ddd")], commands.match_str("d"));
        assert_eq!(Vec::<&Command>::new(), commands.match_str("ad"));
        assert_eq!(vec![&Command::nullary("abc"), &Command::nullary("def"), &Command::nullary("ddd")], commands.match_str(""));
    }

    #[test]
    fn test_completion() {
        let commands: Commands<Completers<_>> = Commands::new()
                                                .add(Command::nullary("foo"))
                                                .add(Command::nullary("fizz"))
                                                .add(Command::nullary("bar"))
                                                .done();

        assert_eq!(
            (3, vec!["bar".into()]),
            commands.complete(" : b ", 0).unwrap(),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into()]),
            commands.complete(":f", 0).unwrap(),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into(), "bar".into()]),
            commands.complete(":", 0).unwrap(),
        );
     }

    #[test]
    fn test_argument_completion() {
        use super::completion::completers::BoolCompleter;

        let completers = Completers::new().add(ArgType::Boolean, Box::new(BoolCompleter));
        let commands = Commands::new()
                        .with_completers(completers)
                        .add(Command::with_arity("abc", ArgType::Boolean, 2))
                        .done();

        assert_eq!(
            (0, vec![]),
            commands.complete(":abc k", 6).unwrap(),
        );

        assert_eq!(
            (5, vec!["true".into()]),
            commands.complete(":abc t", 6).unwrap(),
        );

        assert_eq!(
            (5, vec!["false".into()]),
            commands.complete(":abc f", 6).unwrap(),
        );

        assert_eq!(
            (10, vec!["true".into(), "false".into()]),
            commands.complete(":abc true ", 10).unwrap(),
        );

        assert_eq!(
            (10, vec!["false".into()]),
            commands.complete(":abc true fal", 13).unwrap(),
        );
    }

    #[test]
    fn test_parsing() {
        let commands: Commands<Completers<_>> = Commands::new()
                                                .add(Command::with_arity("foo", ArgType::Number, 2))
                                                .done();

        {
            let text = "foo 1 2";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = " : foo 1 2";
            assert_eq!(
                Ok(CommandCall { command: &Command::with_arity("foo", ArgType::Number, 2), args: vec!["1", "2"] }),
                commands.parse(text),
            );
        }

        {
            let text = ":bar";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }
    }
}

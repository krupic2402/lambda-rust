extern crate rustyline;

use rustyline::completion::{FilenameCompleter, Completer};
use std::str::SplitWhitespace;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ArgType {
    Symbol,
    File,
    Boolean,
    Number,
}

impl ArgType {
    fn get_completer(self) -> Box<dyn Completer> {
        if self == ArgType::File {
            Box::new(FilenameCompleter::default())
        } else {
            Box::new(())
        }
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

pub struct Commands<'commands> {
    commands: Vec<Command<'commands>>,
}

pub struct Builder<'commands> {
    commands: Vec<Command<'commands>>,
}

impl<'commands> Builder<'commands> {
    pub fn add(mut self, command: Command<'commands>) -> Builder<'commands> {
        self.commands.push(command);
        self
    }

    pub fn done(self) -> Commands<'commands> {
        Commands { commands: self.commands }
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

impl<'commands> Commands<'commands> {

    pub fn new() -> Builder<'commands> {
        Builder { commands: vec![] }
    }

    pub fn match_str<'line>(&self, command: &'line str) -> Vec<&Command<'commands>> {
        self.commands.iter().filter(|c| c.name.starts_with(command)).collect()
    }

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

    pub fn parse<'line>(&'commands self, line: &'line str) -> ParseResult<'line, 'commands> {
        match Commands::tokenize(line) {
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

impl<'commands> Completer for Commands<'commands> {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let (command_prefix, position) = match Commands::tokenize(line) {
            Some((command_prefix, start, _)) => (command_prefix, start),
            None => return Ok((0, vec![])),
        };

        let command_candidates = self.match_str(command_prefix);
        if command_candidates.len() == 1 {
            let command = command_candidates[0];
            if command_prefix.len() != command.name.len() {
                return Ok((position, vec![command.name.into()]))
            }

            let completer = command.arg.map(ArgType::get_completer).unwrap_or(Box::new(()));
            return completer.complete(line, pos);
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
        let commands = Commands::new()
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
        let commands = Commands::new()
                            .add(Command::nullary("foo"))
                            .add(Command::nullary("fizz"))
                            .add(Command::nullary("bar"))
                            .done();

        assert_eq!(
            (3, vec!["bar".into()]),
            commands.complete(" : b ", 0).unwrap_or((0, vec!["fail".into()])),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into()]),
            commands.complete(":f", 0).unwrap_or((0, vec!["fail".into()])),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into(), "bar".into()]),
            commands.complete(":", 0).unwrap_or((0, vec!["fail".into()])),
        );
     }

    #[test]
    fn test_parsing() {
        let commands = Commands::new()
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

extern crate rustyline;

use rustyline::completion::Completer;
use std::str::SplitWhitespace;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct Command<'name> {
    pub name: &'name str,
    pub arity: Option<usize>,
}

impl<'name> Command<'name> {
    pub fn new(name: &str) -> Command {
        Command { name, arity: None }
    }

    pub fn with_arity(name: &str, arity: usize) -> Command {
        Command { name, arity: Some(arity) }
    }

    pub fn nullary(name: &str) -> Command {
        Command::with_arity(name, 0)
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
                        Some(n) => args.take(n).collect(),
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
    fn complete(&self, line: &str, _pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let (command, position) = match Commands::tokenize(line) {
            Some((command_prefix, start, _)) => (command_prefix, start),
            None => return Ok((0, vec![])),
        };

        Ok((position, self.match_str(command).into_iter().map(|c| c.name.into()).collect()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matching() {
        let commands = Commands::new()
                            .add(Command::new("abc"))
                            .add(Command::new("def"))
                            .add(Command::new("ddd"))
                            .done();
        assert_eq!(vec![&Command::new("abc")], commands.match_str("a"));
        assert_eq!(vec![&Command::new("def"), &Command::new("ddd")], commands.match_str("d"));
        assert_eq!(Vec::<&Command>::new(), commands.match_str("ad"));
        assert_eq!(vec![&Command::new("abc"), &Command::new("def"), &Command::new("ddd")], commands.match_str(""));
    }

    #[test]
    fn test_completion() {
        let commands = Commands::new()
                            .add(Command::new("foo"))
                            .add(Command::new("fizz"))
                            .add(Command::new("bar"))
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
                            .add(Command::new("foo"))
                            .done();

        {
            let text = "foo 1 2";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = " : foo 1 2";
            assert_eq!(
                Ok(CommandCall { command: &Command::new("foo"), args: vec!["1", "2"] }),
                commands.parse(text),
            );
        }

        {
            let text = ":bar";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }
    }
}

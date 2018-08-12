extern crate rustyline;

use rustyline::completion::Completer;
use std::str::SplitWhitespace;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct Command<'name>(pub &'name str);

pub struct Commands<'names> {
    commands: Vec<Command<'names>>,
}

pub struct Builder<'names> {
    commands: Vec<Command<'names>>,
}

impl<'names> Builder<'names> {
    pub fn add(mut self, command: Command<'names>) -> Builder<'names> {
        self.commands.push(command);
        self
    }

    pub fn done(self) -> Commands<'names> {
        Commands { commands: self.commands }
    }
}

pub const COMMAND_PREFIX: &'static str = ":";

#[derive(Debug, PartialEq)]
pub struct InvalidCommand<'line>(&'line str);

impl<'line> Display for InvalidCommand<'line> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Invalid command: {}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandCall<'line> {
    pub command: &'line str,
    pub args: Vec<&'line str>,
}

impl<'line> Display for CommandCall<'line> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", COMMAND_PREFIX, self.command)?;
        for arg in self.args.iter() {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}

impl<'names> Commands<'names> {
    pub fn new() -> Builder<'names> {
        Builder { commands: vec![] }
    }

    pub fn match_str<'line>(&self, command: &'line str) -> Vec<&Command<'names>> {
        self.commands.iter().filter(|c| c.0.starts_with(command)).collect()
    }

    fn tokenize<'line>(line: &'line str) -> Option<(&'line str, usize, SplitWhitespace<'line>)> {
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

    pub fn parse<'line>(&self, line: &'line str) -> Result<CommandCall<'line>, InvalidCommand<'line>> {
        let (command, _, args) = Commands::tokenize(line)
                                .filter(|c1| self.commands.iter().any(|c| c.0 == c1.0))
                                .ok_or(InvalidCommand(line))?;
        Ok(CommandCall { command, args: args.collect() })
    }
}

impl<'names> Completer for Commands<'names> {
    fn complete(&self, line: &str, _pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let (command, position) = match Commands::tokenize(line) {
            Some((command_prefix, start, _)) => (command_prefix, start),
            None => return Ok((0, vec![])),
        };

        Ok((position, self.match_str(command).into_iter().map(|c| c.0.into()).collect()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matching() {
        let commands = Commands::new()
                            .add(Command("abc"))
                            .add(Command("def"))
                            .add(Command("ddd"))
                            .done();
        assert_eq!(vec![&Command("abc")], commands.match_str("a"));
        assert_eq!(vec![&Command("def"), &Command("ddd")], commands.match_str("d"));
        assert_eq!(Vec::<&Command>::new(), commands.match_str("ad"));
        assert_eq!(vec![&Command("abc"), &Command("def"), &Command("ddd")], commands.match_str(""));
    }

    #[test]
    fn test_completion() {
        let commands = Commands::new()
                            .add(Command("foo"))
                            .add(Command("fizz"))
                            .add(Command("bar"))
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
                            .add(Command("foo"))
                            .done();

        {
            let text = "foo 1 2";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = " : foo 1 2";
            assert_eq!(
                Ok(CommandCall { command: "foo", args: vec!["1", "2"] }),
                commands.parse(text),
            );
        }

        {
            let text = ":bar";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }
    }
}

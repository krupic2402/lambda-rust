extern crate rustyline;

use rustyline::completion::{extract_word, Completer};
use std::cmp::min;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ArgType {
    Symbol,
    File,
    Boolean,
    Number,
    Command,
}

use completion::{self, CompleterProvider, Completers};

impl Default for Completers<ArgType> {
    fn default() -> Self {
        Completers::new()
            .add(ArgType::Boolean, Box::new(completion::completers::BoolCompleter))
            .add(ArgType::File, Box::<rustyline::completion::FilenameCompleter>::default())
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct Command<'name> {
    pub name: &'name str,
    pub arities: Vec<usize>,
    pub arg: Option<ArgType>,
}

impl<'name> Command<'name> {
    pub fn new(name: &str, arg: ArgType) -> Command {
        Command { name, arities: vec![], arg: Some(arg) }
    }

    pub fn with_arities(name: &str, arg: ArgType, arities: Vec<usize>) -> Command {
        Command { name, arities, arg: Some(arg) }
    }

    pub fn unary(name: &str, arg: ArgType) -> Command {
        Command { name, arities: vec![1], arg: Some(arg) }
    }

    pub fn nullary(name: &str) -> Command {
        Command { name, arities: vec![0], arg: None }
    }

    pub fn write_help(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        writeln!(f, "USAGE:")?;
        let arg = self.arg.map_or_else(|| "arg".into() , |arg_type| format!("{:?}", arg_type));
        if !self.arities.is_empty() {
            for arity in &self.arities {
                write!(f, "\t:{}", self.name)?;
                for _ in 0..*arity {
                    write!(f, " {}", arg)?;
                }
                writeln!(f)?;
            }
            Ok(())
        } else {
            writeln!(f, "\t:{} [{}...]", self.name, arg)
        }
    }
}

pub struct Commands<'commands, T: CompleterProvider<ArgType>> {
    commands: Vec<Command<'commands>>,
    completers: Option<T>
}

pub struct Builder<'commands, T: CompleterProvider<ArgType>> {
    commands: Vec<Command<'commands>>,
    completers: Option<T>,
    help: bool,
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

    pub fn with_help(mut self) -> Builder<'commands, T> {
        self.help = true;
        self
    }

    pub fn done(mut self) -> Commands<'commands, T> {
        if self.help {
            let help_command = Command::with_arities(HELP_COMMAND, ArgType::Command, vec![0, 1]);
            self.commands.push(help_command);
        }
        Commands { commands: self.commands, completers: self.completers }
    }
}

pub const COMMAND_PREFIX: &str = ":";
pub const HELP_COMMAND: &str = "help";

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

fn tokenize(line: &str) -> Option<(&str, usize, impl Iterator<Item=&str>)> {
    let start = line.find(COMMAND_PREFIX)? + 1;

    let mut tokens = line[start..].split_whitespace();
    let command = tokens.next();
    let command_prefix = command.unwrap_or("");
    let command_start = command.and_then(|c| line.find(c)).unwrap_or(start);

    Some((command_prefix, command_start, tokens))
}

impl<'commands, T: CompleterProvider<ArgType>> Commands<'commands, T> {

    pub fn new() -> Builder<'commands, T> {
        Builder { commands: vec![], completers: None, help: false }
    }

    fn match_str<'line>(&self, command: &'line str) -> Vec<&Command<'commands>> {
        self.commands.iter().filter(|c| c.name.starts_with(command)).collect()
    }

    fn match_str_exact<'line>(&self, command: &'line str) -> Vec<&Command<'commands>> {
        self.commands.iter().filter(|c| c.name == command).collect()
    }

    pub fn parse<'line>(&'commands self, line: &'line str) -> ParseResult<'line, 'commands> {
        match tokenize(line) {
            Some((command, _, args)) => {
                let candidates = self.match_str(command);
                if candidates.len() == 1 {
                    let command = candidates[0];
                    let args: Vec<_> = args.collect();
                    if !command.arities.is_empty() &&
                        command.arities.iter().find(|a| **a == args.len()).is_none() {
                        return Err(InvalidCommand(line));
                    }

                    Ok(CommandCall { command, args })
                } else {
                    Err(InvalidCommand(line))
                }
            }
            _ => Err(InvalidCommand(line))
        }
    }

    pub fn write_help(&self, f: &mut fmt::Formatter, command_name: Option<&str>) -> fmt::Result {
        let commands: Option<Vec<_>> = command_name.map(|name| self.match_str_exact(name));
        match commands {
            Some(ref commands) if !commands.is_empty() => {
                for command in commands {
                    command.write_help(f)?;
                }
            }
            _ => {
                if let Some(name) = command_name {
                    writeln!(f, "No commands with name: {}", name)?;
                }
                writeln!(f, "Commands:")?;
                for command in &self.commands {
                    writeln!(f, "\t{}", command.name)?;
                }
            }
        }
        Ok(())
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
            } else if command.arg == Some(ArgType::Command) {
                let (position, word_prefix) = extract_word(line, pos, None, &completion::WHITESPACE);
                let command_names = self.match_str(word_prefix).into_iter().map(|c| c.name.into()).collect();

                Ok((position, command_names))
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

        // cursor before command
        assert_eq!(
            (0, vec![]),
            commands.complete(" : b ", 0).unwrap(),
        );

        // cursor after command
        assert_eq!(
            (3, vec!["bar".into()]),
            commands.complete(" : b ", 4).unwrap(),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into()]),
            commands.complete(":f", 2).unwrap(),
        );

        assert_eq!(
            (1, vec!["foo".into(), "fizz".into(), "bar".into()]),
            commands.complete(":", 1).unwrap(),
        );
     }

    #[test]
    fn test_argument_completion() {
        use super::completion::completers::BoolCompleter;

        let completers = Completers::new().add(ArgType::Boolean, Box::new(BoolCompleter));
        let commands = Commands::new()
                        .with_completers(completers)
                        .add(Command::with_arities("abc", ArgType::Boolean, vec![2]))
                        .add(Command::unary("help",  ArgType::Command))
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

        assert_eq!(
            (6, vec!["abc".into(), "help".into()]),
            commands.complete(":help ", 6).unwrap(),
        );
    }

    #[test]
    fn test_parsing() {
        let foo = Command::with_arities("foo", ArgType::Number, vec![1, 2]);
        let commands: Commands<Completers<_>> = Commands::new()
                                                .add(foo.clone())
                                                .done();

        {
            let text = "foo 1 2";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = " : foo 1 2";
            assert_eq!(
                Ok(CommandCall { command: &foo, args: vec!["1", "2"] }),
                commands.parse(text),
            );
        }

        {
            let text = ":foo 8";
            assert_eq!(
                Ok(CommandCall { command: &foo, args: vec!["8"] }),
                commands.parse(text),
            );
        }

        {
            let text = ":foo ";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = ":foo 1 2 3";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }

        {
            let text = ":bar";
            assert_eq!(Err(InvalidCommand(text)), commands.parse(text));
        }
    }
}

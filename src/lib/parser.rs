use ::lexer::Token;
use ::lambda::{Term, Name};

use std::collections::HashMap;
use std::fmt;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    ExpectedToken(&'static str, &'a Token),
    EmptyExpression,
    NotStartOfExpression(&'a Token),
    EOF(Vec<&'static str>),
    UnboundVariable(String),
    TrailingTokens(&'a[Token]),
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;

        match *self {
            ExpectedToken(ref pattern, ref got_token) => {
                write!(f, "Expected {} but got token '{}'", pattern, got_token)
            }
            EmptyExpression => write!(f, "Empty subexpression"),
            NotStartOfExpression(ref got_token) => {
                write!(f, "Invalid token at start of experssion: '{}'", got_token)
            }
            EOF(ref patterns) => {
                write!(f, "Got EOF while expecting any of: {}", patterns.join(", "))
            }
            UnboundVariable(ref variable) => write!(f, "Unbound variable: '{}'", variable),
            TrailingTokens(ref tokens) => {
                write!(f, "Trailing tokens: '{}'",
                       tokens.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "))
            }
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Term, ParseError> {
    let mut symbols = SymbolTable::new();
    let state = ParseState { lambda_depth: 0, symbols: &mut symbols };

    parse_expression(tokens, state)
        .map_err(|e| e.0)
        .and_then(|(term, remaining, _)| {
            if remaining.is_empty() {
                Ok(term)
            } else {
                Err(ParseError::TrailingTokens(remaining))
            }
        })
}

type ParseResult<'a, 'b> = Result<(Term, &'a[Token], ParseState<'b>), (ParseError<'a>, ParseState<'b>)>;
type LambdaDepth = u32;
type SymbolTable = HashMap<String, LambdaDepth>;
struct ParseState<'a> {
    lambda_depth: LambdaDepth,
    symbols: &'a mut SymbolTable,
}

macro_rules! expect_token {
    ($token:pat => $expr:expr, $tokens:expr, $state:expr) => {{
        match $tokens.split_first() {
            Some(($token, rest)) => {
                ($expr, rest)
            }
            None => return Err((ParseError::EOF(vec![stringify!($token)]), $state)),
            _ => return Err((ParseError::ExpectedToken(stringify!($token), $tokens.first().unwrap()), $state)),
        }
    }};
    ($token:pat, $tokens:expr, $state:expr) => {{
        expect_token!($token => (), $tokens, $state)
    }};
}

macro_rules! try_expect_token {
    (($tokens:expr, $rest:pat, $state:expr) { $($token:pat => $found:expr)* } else $failed:block) => {{
        #[allow(unused_variables)]
        match $tokens.split_first() {
            $(
            Some(($token, $rest)) => { $found }
            ),*
            None => return Err((ParseError::EOF(vec![$( stringify!($token) ),*]), $state)),
            _ => $failed
        }
    }};
}


fn parse_expression<'a, 'b>(tokens: &'a[Token], state: ParseState<'b>) -> ParseResult<'a, 'b> {
    use self::Token::*;
    
    try_expect_token! {
        (tokens, rest, state) {
            Identifier(name) => {
                match {state.symbols.get(name)} {
                    Some(&parent) => {
                        let de_bruijn = state.lambda_depth - parent;
                        Ok((Term::variable(Name::bound(de_bruijn)), rest, state))
                    }
                    None => {
                        Ok((Term::variable(Name::free(name.clone())), rest, state))
                    }
                }
            }
            ParenOpen => {
                let tokens = rest;

                let (expr, tokens, state) = try_expect_token! {
                    (tokens, _, state) {
                        Lambda => parse_lambda(tokens, state)?
                    } else {
                        parse_application(tokens, state)?
                    }
                };

                let (_, tokens) = expect_token!(ParenClose, tokens, state);

                Ok((expr, tokens, state))
            }
        } else {
            Err((ParseError::NotStartOfExpression(tokens.first().unwrap()), state))
        }
    }
}

fn parse_application<'a, 'b>(mut tokens: &'a[Token], mut state: ParseState<'b>) -> ParseResult<'a, 'b> {
    let mut expr = None;

    loop {
        match parse_expression(tokens, state) {
            Ok((term, new_tokens, new_state)) => { 
                expr = match expr {
                    Some(t) => Some(Term::apply(t, term)),
                    _ => Some(term),
                };
                state = new_state;
                tokens = new_tokens;
            }
            Err((ParseError::NotStartOfExpression(_), err_state)) => {
                state = err_state;
                break;
            }
            e => return e,
        }
    } 

    match expr {
        Some(term) => Ok((term, tokens, state)),
        _ => Err((ParseError::EmptyExpression, state)),
    }
}

fn parse_lambda<'a, 'b>(tokens: &'a[Token], state: ParseState<'b>) -> ParseResult<'a, 'b> {
    use self::Token::*;
    let (_, tokens) = expect_token!(Lambda, tokens, state);
    let (name, tokens) = expect_token!(Identifier(name) => name.clone(), tokens, state);
    let (_, tokens) = expect_token!(Dot, tokens, state);

    // perform shadowing binding
    let old_binding = state.symbols.insert(name.clone(), state.lambda_depth);
    let state = ParseState { lambda_depth: state.lambda_depth + 1, symbols: state.symbols };

    let (body, tokens, state) = parse_expression(tokens, state)?;

    let state = ParseState { lambda_depth: state.lambda_depth - 1, symbols: state.symbols };
    // recover old binding if present
    if let Some(lambda_depth) = old_binding {
        state.symbols.insert(name, lambda_depth);
    }

    Ok((Term::lambda(body), tokens, state))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_simple_lambda() {
        let lambda = "(Lx.x)";
        let tokens = Token::parse_all(lambda).unwrap();

        assert_eq!(
            Ok(Term::lambda(Term::variable(Name::bound(1)))),
            parse(&tokens),
        );
    }

    #[test]
    fn test_parse_nested_lambda() {
        let lambda = "(Lf.(Lx.(Ly.(f x y))))";
        let tokens = Token::parse_all(lambda).unwrap();

        assert_eq!(
            Ok(Term::lambda(Term::lambda(Term::lambda(
                Term::apply(
                    Term::apply(
                        Term::variable(Name::bound(3)),
                        Term::variable(Name::bound(2)),
                    ),
                    Term::variable(Name::bound(1)),
                )
            )))),
            parse(&tokens),
        );
    }

    #[test]
    fn test_parse_free_variable() {
        let lambda = "a";
        let tokens = Token::parse_all(lambda).unwrap();

        assert_eq!(
            Ok(Term::variable(Name::free("a".into()))),
            parse(&tokens),
        );
    }
}

use ::lexer::Token;
use ::lambda::{Term, Name};

use std::collections::HashMap;

pub struct ParseError;

pub fn parse(tokens: &[Token]) -> Result<Term, ParseError> {
    let mut symbols = SymbolTable::new();
    let state = ParseState { depth: 0, symbols: &mut symbols };
    parse_expression(tokens, state).map(|o| o.0).map_err(|e| e.0)
}

// parsed result + remaining tokens | error
type PartialParseResult<'a, T> = Result<(T, &'a[Token], ParseState<'a>), (ParseError, ParseState<'a>)>;
type Depth = u32;
type SymbolTable = HashMap<String, Depth>;
struct ParseState<'a> {
    depth: Depth,
    symbols: &'a mut SymbolTable,
}

macro_rules! expect_token {
    ($token:pat => $expr:expr, $tokens:expr, $state:expr) => {{
        match $tokens.split_first() {
            Some(($token, rest)) => {
                ($expr, rest)
            }
            _ => return Err((ParseError, $state)),
        }
    }};
    ($token:pat, $tokens:expr, $state:expr) => {{
        expect_token!($token => (), $tokens, $state)
    }};
}

macro_rules! try_expect_token {
    (($tokens:expr, $rest:ident) if $token:pat => $found:block else $failed:block) => {{
        #[allow(unused_variables)]
        match $tokens.split_first() {
            Some(($token, $rest)) => $found
            _ => $failed
        }
    }};
}


fn parse_expression<'a>(tokens: &'a[Token], state: ParseState<'a>) -> PartialParseResult<'a, Term> {
    use self::Token::*;
    
    try_expect_token! {
        (tokens, rest) if Identifier(name) => {
            match {state.symbols.get(name)} {
                Some(&parent) => {
                    let de_bruijn = state.depth - parent;
                    Ok((Term::variable(Name::new(de_bruijn)), rest, state))
                }
                None => {
                    Err((ParseError, state)) 
                }
            }
        } else {
            let (_, tokens) = expect_token!(ParenOpen, tokens, state);
            
            let result = try_expect_token! {
                (tokens, rest) if Lambda => {
                    parse_lambda(tokens, state)?
                } else {
                    parse_application(tokens, state)?
                }
            };

            let (_, tokens) = expect_token!(ParenClose, tokens, result.2);

            Ok(result)
        }
    }
}

fn parse_application<'a>(mut tokens: &'a[Token], mut state: ParseState<'a>) -> PartialParseResult<'a, Term> {
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
            Err((_, err_state)) => {
                state = err_state;
                break;
            }
        }
    } 

    match expr {
        Some(term) => Ok((term, tokens, state)),
        _ => Err((ParseError, state)),
    }
}

fn parse_lambda<'a>(tokens: &'a[Token], state: ParseState<'a>) -> PartialParseResult<'a, Term> {
    use self::Token::*;
    let (_, tokens) = expect_token!(Lambda, tokens, state);
    let (name, tokens) = expect_token!(Identifier(name) => name.clone(), tokens, state);
    let (_, tokens) = expect_token!(Dot, tokens, state);

    let old_binding = state.symbols.insert(name.clone(), state.depth);

    let (body, tokens, state) = {
        let state = ParseState { depth: state.depth + 1, symbols: state.symbols };
        let (body, tokens, state) = parse_expression(tokens, state)?;
        let state = ParseState { depth: state.depth - 1, symbols: state.symbols };
        (body, tokens, state) 
    };

    if let Some(depth) = old_binding {
        state.symbols.insert(name, depth);
    }

    Ok((Term::lambda(body), tokens, state))
}

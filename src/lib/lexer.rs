use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token {
    ParenOpen,
    ParenClose,
    Lambda,
    Dot,
    Identifier(String),
    Let,
    DefineReduce,
    DefineSuspend,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Token::*;

        match *self {
            ParenOpen => write!(f, "("),
            ParenClose => write!(f, ")"),
            Lambda => write!(f, "λ"),
            Dot => write!(f, "."),
            Identifier(ref name) => write!(f, "{}", name),
            Let => write!(f, "let"),
            DefineReduce => write!(f, "="),
            DefineSuspend => write!(f, ":="),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTokenError(pub String);

impl Token {
    pub fn parse_all(s: &str) -> Result<Vec<Token>, ParseTokenError> {
        use self::Token::*;

        let mut tokens = vec![];
        let mut iterator = s.chars().peekable(); 

        while let Some(c) = iterator.next() {
            if c.is_whitespace() { continue; }
            match c {
                '(' => tokens.push(ParenOpen),
                ')' => tokens.push(ParenClose),
                'λ' | 'L' => tokens.push(Lambda),
                '.' => tokens.push(Dot),
                '=' => tokens.push(DefineReduce),
                ':' => {
                    match iterator.next() {
                        Some('=') => tokens.push(DefineSuspend),
                        _ => return Err(ParseTokenError(format!("Invalid token: :{}", c))),
                    }
                },
                c if c.is_ascii_alphanumeric()  => {
                    let mut word: String = String::new();
                    word.push(c);

                    while let Some(&c) = iterator.peek() {
                        if !c.is_ascii_alphanumeric() { break; }
                        word.push(iterator.next().unwrap());
                    }

                    if word == "let" {
                        tokens.push(Let);
                    } else {
                        tokens.push(Identifier(word));
                    }
                }
                _ => return Err(ParseTokenError(format!("Invalid token: {}", c))),
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use self::Token::*;

    #[test]
    fn test_parse_tokens_correct() {
        assert_eq!(
            Ok(vec![ParenOpen, Lambda, Identifier("x".into()), Dot, Identifier("x".into()), ParenClose]),
            Token::parse_all("  (Lx.  x  ) ")
        );
    }
        
    #[test]
    fn test_parse_tokens_invalid() {
        assert_eq!(
            Err(ParseTokenError("Invalid token: [".into())),
            Token::parse_all("[Lx.x]"),
        );
    }

    #[test]
    fn test_parse_tokens_empty() {
        assert_eq!(
            Ok(vec![]),
            Token::parse_all(" "),
        );
    }

    #[test]
    fn test_parse_tokens_let_statement() {
        assert_eq!(
            Ok(vec![Let, Identifier("I".into()), DefineReduce,
                ParenOpen, Lambda, Identifier("x".into()), Dot, Identifier("x".into()), ParenClose]),
            Token::parse_all("let I = (Lx.x)"),
        );

        assert_eq!(
            Ok(vec![Let, Identifier("I".into()), DefineSuspend,
                ParenOpen, Lambda, Identifier("x".into()), Dot, Identifier("x".into()), ParenClose]),
            Token::parse_all("let I := (Lx.x)"),
        );
    }

    #[test]
    fn test_parse_back_displayed() {
        let tokens = vec![
            ParenOpen, ParenClose, Lambda, Dot, Let, DefineReduce, DefineSuspend, Identifier("x".into())
        ];

        let text = tokens.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ");

        assert_eq!(Ok(tokens), Token::parse_all(&text));
    }
}

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token {
    ParenOpen,
    ParenClose,
    Lambda,
    Dot,
    Identifier(String),
    Let,
    Define,
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
            Define => write!(f, "="),
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
                '=' => tokens.push(Define),
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
            Ok(vec![Let, Identifier("I".into()), Define,
                ParenOpen, Lambda, Identifier("x".into()), Dot, Identifier("x".into()), ParenClose]),
            Token::parse_all("let I = (Lx.x)"),
        );
    }
}

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token {
    ParenOpen,
    ParenClose,
    Lambda,
    Dot,
    Identifier(String),
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
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTokenError;

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
                c if c.is_ascii_alphanumeric()  => {
                    let mut identifier: String = String::new();
                    identifier.push(c);

                    loop {
                        let c = {
                            let c = iterator.peek();
                            if c.is_none() { break; }
                            *c.unwrap()
                        };

                        if c.is_ascii_alphanumeric() {
                            identifier.push(iterator.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    tokens.push(Identifier(identifier));
                }
                _ => return Err(ParseTokenError),
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
    fn test_parse_all() {

        assert_eq!(
            Ok(vec![ParenOpen, Lambda, Identifier("x".into()), Dot, Identifier("x".into()), ParenClose]),
            Token::parse_all("  (Lx.  x  ) ")
        );
        
        assert_eq!(
            Err(ParseTokenError),
            Token::parse_all("[Lx.x]"),
        );

        assert_eq!(
            Ok(vec![]),
            Token::parse_all(" "),
        );
    }
}

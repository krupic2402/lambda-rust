use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Name {
    depth: u32
}

impl Name {
    pub fn new(depth: u32) -> Name {
        Name { depth }
    }

    fn rebind(&mut self, deepen_by: i32) {
        self.depth = (self.depth as i32 + deepen_by) as u32;
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "↑{}", self.depth)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Lambda {
        body: Box<Term>
    },
    Application {
        applicand: Box<Term>,
        argument: Box<Term>
    },
    Variable {
        name: Name
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Strategy { NormalOrder, ApplicativeOrder }

#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    NormalForm(Term),
    PossiblyReducible(Term)
}

impl EvalResult {
    pub fn unwrap(self) -> Term {
        use self::EvalResult::*;

        match self {
            NormalForm(t) => t,
            PossiblyReducible(t)  => t
        }
    }

    pub fn map<F: FnOnce(Term) -> Term>(self, f: F) -> EvalResult {
        use self::EvalResult::*;

        match self {
            NormalForm(t) => NormalForm(f(t)),
            PossiblyReducible(t)  => PossiblyReducible(f(t))
        }
    }
}

impl Term {
    pub fn variable<T: Into<Name>>(name: T) -> Term {
        Term::Variable { name: name.into() }
    }

    pub fn lambda(body: Term) -> Term {
        Term::Lambda { body: Box::new(body) }
    }

    pub fn apply(applicand: Term, argument: Term) -> Term {
        Term::Application { applicand: Box::new(applicand), argument: Box::new(argument) }
    }

    fn rebind_free(&mut self, deepen_by: i32, depth: u32) {
        match self {
            Term::Variable { ref mut name } => {
                if name.depth > depth {
                    name.rebind(deepen_by);
                }
            }
            Term::Application { ref mut applicand, ref mut argument } => {
                applicand.rebind_free(deepen_by, depth);
                argument.rebind_free(deepen_by, depth);
            }
            Term::Lambda { ref mut body } => {
                body.rebind_free(deepen_by, depth + 1);
            }
        }
    }

    fn substitute(self, depth: u32, deepen_by: i32, mut with: Term) -> Term {
        match self {
            Term::Variable { name } => {
                if name.depth == depth {
                    with.rebind_free(deepen_by, 0);
                    with
                } else {
                    Term::variable(name)
                }
            }
            Term::Application { applicand, argument } => {
                let applicand = applicand.substitute(depth, deepen_by, with.clone());
                let argument = argument.substitute(depth, deepen_by, with);
                Term::apply(applicand, argument)
            }
            Term::Lambda { body } => {
                Term::lambda(body.substitute(depth + 1, deepen_by + 1, with))
            }
        }
    }

    pub fn reduce(self, strategy: Strategy) -> EvalResult {
        use self::EvalResult::*;

        match strategy {
            Strategy::NormalOrder => {
                match self {
                    v @ Term::Variable { .. } =>
                        NormalForm(v),
                    Term::Lambda { body } =>
                        body.reduce(strategy).map(Term::lambda),
                    Term::Application { applicand, argument } => {
                        let applicand = *applicand;
                        let argument = *argument;
                        if let Term::Lambda { body } = applicand {
                            let mut body = body.substitute(1, 1, argument);
                            body.rebind_free(-1, 0);
                            return PossiblyReducible(body);
                        } else {
                            let head = applicand.reduce(strategy);

                            match head {
                                PossiblyReducible(_) =>
                                    return head.map(|t| Term::apply(t, argument)),
                                NormalForm(head) =>
                                    return argument.reduce(strategy).map(|t| Term::apply(head, t))
                            }
                        }
                    }
                } 
            }
            _ => unimplemented!()
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter, depth: u32, symbols: &mut Vec<String>) -> fmt::Result {
        use self::Term::*;

        match *self {
            Variable { ref name } => {
                if name.depth <= depth {
                    return write!(f, "{}", symbols[(depth - name.depth) as usize]);
                } else {
                    return write!(f, "{}", name);
                }
            }
            Application { ref applicand, ref argument } => {
                write!(f, "(")?;
                applicand.fmt(f, depth, symbols)?;
                write!(f, " ")?;
                argument.fmt(f, depth, symbols)?;
                return write!(f, ")");
            }
            Lambda { ref body } => {
                let name = format!("x{}", depth);
                write!(f, "(λ{}.", name)?;
                assert_eq!(symbols.len(), depth as usize);
                symbols.push(name);

                body.fmt(f, depth + 1, symbols)?;
                symbols.pop();
                return write!(f, ")");
            }
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut symbols = vec![];
        self.fmt(f, 0, &mut symbols)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reduction_simple() {
        let term = Term::lambda(Term::apply(
            Term::lambda(Term::variable(Name::new(1))),
            Term::variable(Name::new(1))
        ));

        let result = term.reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::PossiblyReducible(Term::lambda(Term::variable(Name::new(1)))),
            result
        );

        let result = result.unwrap().reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::NormalForm(Term::lambda(Term::variable(Name::new(1)))),
            result
        );
    }

    #[test]
    fn test_reduction_complex() {
        let term = Term::apply(
            Term::lambda(Term::lambda(Term::lambda(
                Term::apply(
                    Term::apply(
                        Term::variable(Name::new(3)),
                        Term::variable(Name::new(2)),
                    ),
                    Term::variable(Name::new(1)),
                )
            ))),
            Term::lambda(Term::lambda(
                Term::variable(Name::new(2)),
            )),
        );

        let result = term.reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::PossiblyReducible(
                Term::lambda(Term::lambda(
                    Term::apply(
                        Term::apply(
                            Term::lambda(Term::lambda(
                                Term::variable(Name::new(2)),
                            )),
                            Term::variable(Name::new(2)),
                        ),
                        Term::variable(Name::new(1)),
                    )
                ))
             ),
             result
        );

        let result = result.unwrap().reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::PossiblyReducible(
                Term::lambda(Term::lambda(
                    Term::apply(
                        Term::lambda(
                            Term::variable(Name::new(3)),
                        ),
                        Term::variable(Name::new(1)),
                    )
                ))
            ),
            result
        );

        let result = result.unwrap().reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::PossiblyReducible(
                Term::lambda(Term::lambda(
                    Term::variable(Name::new(2)),
                ))
            ),
            result
        );

        let result = result.unwrap().reduce(Strategy::NormalOrder);
        assert_eq!(
            EvalResult::NormalForm(
                Term::lambda(Term::lambda(
                    Term::variable(Name::new(2)),
                ))
            ),
            result
        );
    }
}

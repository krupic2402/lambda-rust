
use std::fmt;

const LAMBDA: &'static str = "λ";
const ALPHA: &'static str = "α";
const BETA: &'static str = "β";
const ETA: &'static str = "η";

//type Name = String;

#[derive(Debug, PartialEq, Clone)]
struct Name {
    depth: u32
}

impl Name {
    fn new(depth: u32) -> Name {
        Name { depth }
    }

    fn rebind(&mut self, deepen_by: u32) {
        self.depth += deepen_by;
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "↑{}", self.depth)
    }
}

#[derive(Debug, Clone)]
enum Term {
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
enum Strategy { NormalOrder, ApplicativeOrder }

#[derive(Debug, Clone)]
enum EvalResult {
    NormalForm(Term),
    PossiblyReducible(Term)
}

impl EvalResult {
    fn unwrap(self) -> Term {
        use EvalResult::*;

        match self {
            NormalForm(t) => t,
            PossiblyReducible(t)  => t
        }
    }

    fn map<F: FnOnce(Term) -> Term>(self, f: F) -> EvalResult {
        use EvalResult::*;

        match self {
            NormalForm(t) => NormalForm(f(t)),
            PossiblyReducible(t)  => PossiblyReducible(f(t))
        }
    }
}

impl Term {
    fn variable<T: Into<Name>>(name: T) -> Term {
        Term::Variable { name: name.into() }
    }

    fn lambda(body: Term) -> Term {
        Term::Lambda { body: Box::new(body) }
    }

    fn apply(applicand: Term, argument: Term) -> Term {
        Term::Application { applicand: Box::new(applicand), argument: Box::new(argument) }
    }

    fn rebind_free(&mut self, deepen_by: u32, depth: u32) {
        match *self {
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
                body.rebind_free(deepen_by + 1, depth + 1);
            }
        }
    }

    fn substitute(self, depth: u32, deepen_by: u32, mut with: Term) -> Term {
        println!("substitute {} with {} in {}", depth, with, self);
        match self {
            Term::Variable { name } => {
                if name.depth == depth {
                    with.rebind_free(deepen_by, 0);
                    return with;
                } else {
                    return Term::variable(name);
                }
            }
            Term::Application { applicand, argument } => {
                let applicand = applicand.substitute(depth, deepen_by, with.clone());
                let argument = argument.substitute(depth, deepen_by, with);
                return Term::apply(applicand, argument);
            }
            Term::Lambda { body } => {
                return Term::lambda(body.substitute(depth + 1, deepen_by + 1, with));
            }
        }
    }

    fn reduce(self, strategy: Strategy) -> EvalResult {
        use EvalResult::*;

        match strategy {
            Strategy::NormalOrder => {
                match self {
                    v @ Term::Variable { .. } =>
                        return NormalForm(v),
                    Term::Lambda { body } =>
                        return body.reduce(strategy).map(Term::lambda),
                    Term::Application { applicand, argument } => {
                        let applicand = *applicand;
                        let argument = *argument;
                        if let Term::Lambda { body } = applicand {
                            return PossiblyReducible(body.substitute(1, 0, argument));
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
        use Term::*;
        
        match self {
            &Variable { ref name } => {
                if name.depth <= depth {
                    return write!(f, "{}", symbols[(depth - name.depth) as usize]);
                } else {
                    return write!(f, "{}", name);
                }
            }
            &Application { ref applicand, ref argument } => {
                write!(f, "(")?;
                applicand.fmt(f, depth, symbols)?;
                write!(f, " ")?;
                argument.fmt(f, depth, symbols)?;
                return write!(f, ")");
            }
            &Lambda { ref body } => {
                let name = format!("x{}", depth);
                write!(f, "({}{}.", LAMBDA, name)?;
                assert_eq!(symbols.len(), depth as usize);
                symbols.push(name);
                
                body.fmt(f, depth + 1, symbols)?;
                return write!(f, ")");
            }
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) ->  fmt::Result {
        let mut symbols = vec![];
        self.fmt(f, 0, &mut symbols)
    }
}

fn main() {
    let term = {
        Term::apply(
            Term::lambda(
                Term::apply(
                    Term::variable(Name::new(1)),
                    Term::lambda(
                        Term::apply(
                            Term::variable(Name::new(1)),
                            Term::variable(Name::new(2))
                        )
                    )
                )
            ),
            Term::variable(Name::new(99))
        )
    };
    println!("{}", term);

    println!("{}", term.reduce(Strategy::NormalOrder).unwrap().reduce(Strategy::NormalOrder).unwrap());
}

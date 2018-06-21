
use std::fmt;
use std::convert;

const LAMBDA: &'static str = "λ";
const ALPHA: &'static str = "α";
const BETA: &'static str = "β";
const ETA: &'static str = "η";

//type Name = String;

#[derive(Debug, PartialEq, Clone)]
struct Name {
    name: String,
    id: u32
}

static mut ID_COUNTER: u32 = 0;

impl Name {
    fn fresh(&self) -> Name {
        unsafe {
            ID_COUNTER += 1;
            Name { id: ID_COUNTER, name: self.name.clone() }
        }
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.name, self.id)
    }
}

impl<'a> convert::From<&'a str> for Name {
    fn from(name: &str) -> Name {
        unsafe {
            ID_COUNTER += 1;
            Name { name: name.into(), id: ID_COUNTER }
        }
    }
}

#[derive(Debug, Clone)]
enum Term {
    Lambda {
        bound_variable: Name,
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

    fn lambda<T: Into<Name>>(variable: T, body: Term) -> Term {
        Term::Lambda { bound_variable: variable.into(), body: Box::new(body) }
    }

    fn apply(applicand: Term, argument: Term) -> Term {
        Term::Application { applicand: Box::new(applicand), argument: Box::new(argument) }
    }

    fn rename_bound(&mut self, from: &Name, to: &Name) {
        match *self {
            Term::Variable { ref mut name } => {
                if *name == *from {
                    *name = to.clone();
                }
            }
            Term::Application { ref mut applicand, ref mut argument } => {
                applicand.rename_bound(from, to);
                argument.rename_bound(from, to);
            }
            Term::Lambda { ref bound_variable, ref mut body } => {
                if *bound_variable != *from {
                    body.rename_bound(from, to);
                }
            }
        }
    }

    fn alpha_rename(&mut self) {
        match *self {
            Term::Lambda { ref mut bound_variable, ref mut body } => {
                let old = bound_variable.clone();
                *bound_variable = bound_variable.fresh();
                body.rename_bound(&old, &bound_variable);
            }
            _ => {}
        }
    }

    fn substitute(self, what: &Name, with: Term) -> Term {
        println!("substitute {} with {} in {}", what, with, self);
        match self {
            Term::Variable { name } => {
                if name == *what {
                    return with;
                } else {
                    return Term::variable(name);
                }
            }
            Term::Application { applicand, argument } => {
                let applicand = applicand.substitute(what, with.clone());
                let argument = argument.substitute(what, with);
                return Term::apply(applicand, argument);
            }
            Term::Lambda { bound_variable, body } => {
                if bound_variable == *what {
                    return Term::lambda(bound_variable, *body);
                } else {
                    let mut lambda = Term::lambda(bound_variable, *body);
                    lambda.alpha_rename();

                    if let Term::Lambda { bound_variable, body } = lambda {
                        return Term::lambda(bound_variable, body.substitute(what, with));
                    } else {
                        unreachable!();
                    }
                }
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
                    Term::Lambda { bound_variable, body } =>
                        return body.reduce(strategy)
                                   .map(|t| Term::lambda(bound_variable, t)),
                    Term::Application { applicand, argument } => {
                        let applicand = *applicand;
                        let argument = *argument;
                        if let Term::Lambda { bound_variable, body } = applicand {
                            return PossiblyReducible(body.substitute(&bound_variable, argument));
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
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) ->  fmt::Result {
        match self {
            &Term::Variable { ref name } =>
                write!(f, "{}", name),
            &Term::Application { ref applicand, ref argument } =>
                write!(f, "({} {})", applicand, argument),
            &Term::Lambda { ref bound_variable, ref body } =>
                write!(f, "({}{}.{})", LAMBDA, bound_variable, body),
        }
    }
}

fn main() {
    let mut term = {
        let x: Name = "x".into();
        let y: Name = "y".into();
        Term::apply(
            Term::lambda(
                x.clone(),
                Term::apply(
                    Term::variable(x.clone()),
                    Term::lambda(
                        x.clone(),
                        Term::apply(
                            Term::variable(x.clone()),
                            Term::variable(x.clone())
                        )
                    )
                )
            ),
            Term::variable(y)
        )
    };
    println!("{}", term);
    
    if let Term::Application { ref mut applicand, .. } = term {
        applicand.alpha_rename();
    }
    println!("{}", term);

    println!("{:?}", term.reduce(Strategy::NormalOrder).unwrap().reduce(Strategy::NormalOrder));
}

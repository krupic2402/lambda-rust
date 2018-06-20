
const LAMBDA: &'static str = "λ";
const ALPHA: &'static str = "α";
const BETA: &'static str = "β";
const ETA: &'static str = "η";

type Name = String;

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
}

use std::fmt;

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
    println!("{}", Term::apply(Term::lambda("x", Term::variable("x")), Term::variable("y")));
}

extern crate lambda_rust;

use lambda_rust::lambda::*;

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

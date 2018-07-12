mod abstraction;
mod application;
mod variable;
pub use self::abstraction::*;
pub use self::application::*;
pub use self::variable::*;

use cst::Expression as CSTExpression;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Variable(Variable),
    Abstraction(Abstraction),
    Application(Application),
}

impl Expression {
    pub fn from_cst<'a>(value: &'a CSTExpression, scopes: &mut Vec<&'a str>) -> Expression {
        match value {
            CSTExpression::Variable(variable) => {
                Expression::Variable(Variable::from_cst(variable, scopes))
            }

            CSTExpression::Abstraction(abstraction) => {
                Expression::Abstraction(Abstraction::from_cst(abstraction, scopes))
            }

            CSTExpression::Application(application) => {
                if application.expressions.len() > 1 {
                    Expression::Application(Application::from_cst(application, scopes))
                } else if application.expressions.len() == 1 {
                    Expression::from_cst(&application.expressions[0], scopes)
                } else {
                    panic!()
                }
            }

            CSTExpression::Number(number) => Expression::Abstraction(Abstraction::from(number)),

            CSTExpression::Character(character) => {
                Expression::Abstraction(Abstraction::from(character))
            }
        }
    }

    pub fn is_reducible(&self) -> bool {
        match self {
            Expression::Variable(..) => false,
            Expression::Abstraction(..) => false,
            Expression::Application(application) => application.is_reducible(),
        }
    }

    pub fn evaluate(mut self) -> Expression {
        while self.is_reducible() {
            self = self.evaluate1();
        }
        self
    }

    fn evaluate1(self) -> Expression {
        if let Expression::Application(application) = self {
            application.evaluate1()
        } else {
            self
        }
    }

    fn shifted(self, d: isize, c: usize) -> Expression {
        match self {
            Expression::Variable(variable) => Expression::Variable(variable.shifted(d, c)),
            Expression::Abstraction(abstraction) => {
                Expression::Abstraction(abstraction.shifted(d, c))
            }
            Expression::Application(application) => {
                Expression::Application(application.shifted(d, c))
            }
        }
    }

    fn substituted(self, j: usize, term: Expression) -> Expression {
        match self {
            Expression::Variable(variable) => variable.substituted(j, term),
            Expression::Abstraction(abstraction) => {
                Expression::Abstraction(abstraction.substituted(j, term))
            }
            Expression::Application(application) => {
                Expression::Application(application.substituted(j, term))
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Variable(variable) => variable.fmt(f),
            Expression::Abstraction(abstraction) => abstraction.fmt(f),
            Expression::Application(application) => application.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shift() {
        let expected = Expression::Variable(Variable::new(1, "x"));
        let result = Expression::Variable(Variable::new(0, "x")).shifted(1, 0);
        assert_eq!(expected, result);

        let expected = Expression::Variable(Variable::new(0, "x"));
        let result = Expression::Variable(Variable::new(0, "x")).shifted(1, 1);
        assert_eq!(expected, result);

        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(2, "y")),
        ));
        let result = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(1, "y")),
        )).shifted(1, 0);
        assert_eq!(expected, result);

        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(0, "x")),
        ));
        let result = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(0, "x")),
        )).shifted(1, 0);
        assert_eq!(expected, result);

        let expected = Expression::Application(Application::new(
            Expression::Variable(Variable::new(1, "x")),
            Expression::Variable(Variable::new(2, "y")),
        ));
        let result = Expression::Application(Application::new(
            Expression::Variable(Variable::new(0, "x")),
            Expression::Variable(Variable::new(1, "y")),
        )).shifted(1, 0);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_substitute() {
        let expected = Expression::Variable(Variable::new(None, "a"));
        let result = Expression::Variable(Variable::new(0, "x"))
            .substituted(0, Expression::Variable(Variable::new(None, "a")));
        assert_eq!(expected, result);

        let expected = Expression::Variable(Variable::new(1, "x"));
        let result = Expression::Variable(Variable::new(1, "x"))
            .substituted(0, Expression::Variable(Variable::new(None, "a")));
        assert_eq!(expected, result);

        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(None, "a")),
        ));
        let result = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(1, "y")),
        )).substituted(0, Expression::Variable(Variable::new(None, "a")));
        assert_eq!(expected, result);

        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(0, "x")),
        ));
        let result = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Variable(Variable::new(0, "x")),
        )).substituted(0, Expression::Variable(Variable::new(None, "a")));
        assert_eq!(expected, result);

        let expected = Expression::Application(Application::new(
            Expression::Variable(Variable::new(0, "x")),
            Expression::Variable(Variable::new(None, "a")),
        ));
        let result = Expression::Application(Application::new(
            Expression::Variable(Variable::new(0, "x")),
            Expression::Variable(Variable::new(1, "y")),
        )).substituted(1, Expression::Variable(Variable::new(None, "a")));
        assert_eq!(expected, result);
    }
}

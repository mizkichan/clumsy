mod abstraction;
mod application;
mod variable;
pub use self::abstraction::*;
pub use self::application::*;
pub use self::variable::*;

use cst;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Variable(Variable),
    Abstraction(Abstraction),
    Application(Application),
}

impl Expression {
    pub fn assign_indices<'a>(&'a mut self, table: &mut HashMap<&'a str, usize>) {
        match self {
            Expression::Variable(variable) => variable.assign_indices(table),
            Expression::Abstraction(abstraction) => abstraction.assign_indices(table),
            Expression::Application(application) => application.assign_indices(table),
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

impl<'a> From<&'a cst::Expression> for Expression {
    fn from(value: &cst::Expression) -> Expression {
        let mut result = match value {
            cst::Expression::Variable(cst::VariableExpression { identifier }) => {
                Expression::Variable(Variable::from(identifier))
            }

            cst::Expression::Abstraction(abstraction) => {
                Expression::Abstraction(Abstraction::from(abstraction))
            }

            cst::Expression::Application(application) => Expression::from(application),

            cst::Expression::Number(number) => Expression::Abstraction(Abstraction::from(number)),
        };

        result.assign_indices(&mut HashMap::new());
        result
    }
}

impl<'a> From<&'a cst::Program> for Expression {
    fn from(value: &cst::Program) -> Expression {
        let cst::Program(statements) = value;

        let mut iter = statements.iter();
        match iter.next_back() {
            Some(cst::Statement::Expression(cst::ExpressionStatement { expression: result })) => {
                let mut result = iter.rfold(Expression::from(result), |result, statement| {
                    match statement {
                        cst::Statement::Expression(..) => unimplemented!(),
                        cst::Statement::Let(cst::LetStatement {
                            variable: cst::Identifier(variable),
                            expression,
                        }) => Expression::Application(Application::new(
                            Expression::Abstraction(Abstraction::new(variable.to_owned(), result)),
                            expression,
                        )),
                    }
                });
                result.assign_indices(&mut HashMap::new());
                result
            }

            Some(cst::Statement::Let(..)) => unimplemented!(),
            None => unreachable!(),
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
    fn translate_abstraction() {
        let result = Expression::from(&cst::Expression::from(cst::AbstractionExpression::new(
            vec![cst::Identifier::new("x"), cst::Identifier::new("x")],
            cst::VariableExpression::new(cst::Identifier::new("x")),
        )));

        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Abstraction(Abstraction::new(
                "x",
                Expression::Variable(Variable::new(Some(0), "x")),
            )),
        ));
        assert_eq!(expected, result);

        let b = Expression::from(&cst::Expression::from(cst::AbstractionExpression::new(
            vec![cst::Identifier::new("x")],
            cst::ApplicationExpression::new(vec![
                cst::Expression::from(cst::AbstractionExpression::new(
                    vec![cst::Identifier::new("x")],
                    cst::VariableExpression::new(cst::Identifier::new("x")),
                )),
                cst::Expression::from(cst::VariableExpression::new(cst::Identifier::new("x"))),
            ]),
        )));
        let expected = Expression::Abstraction(Abstraction::new(
            "x",
            Expression::Application(Application::new(
                Expression::Abstraction(Abstraction::new(
                    "x",
                    Expression::Variable(Variable::new(Some(0), "x")),
                )),
                Expression::Variable(Variable::new(Some(0), "x")),
            )),
        ));
        assert_eq!(expected, b);
    }

    #[test]
    fn translate_application() {
        let a = Expression::from(&cst::Expression::from(cst::ApplicationExpression::new(
            vec![
                cst::Expression::from(cst::VariableExpression::new(cst::Identifier::new("a"))),
                cst::Expression::from(cst::VariableExpression::new(cst::Identifier::new("b"))),
                cst::Expression::from(cst::VariableExpression::new(cst::Identifier::new("c"))),
            ],
        )));
        let expected = Expression::Application(Application::new(
            Expression::Application(Application::new(
                Expression::Variable(Variable::new(None, "a")),
                Expression::Variable(Variable::new(None, "b")),
            )),
            Expression::Variable(Variable::new(None, "c")),
        ));
        assert_eq!(expected, a);
    }

    #[test]
    fn translate_let_statement() {
        let expected = Expression::Application(Application::new(
            Expression::Abstraction(Abstraction::new(
                "id",
                Expression::Variable(Variable::new(0, "id")),
            )),
            Expression::Abstraction(Abstraction::new(
                "x",
                Expression::Variable(Variable::new(0, "x")),
            )),
        ));
        let result = Expression::from(&cst::Program(vec![
            cst::Statement::from(cst::LetStatement::new(
                cst::Identifier::new("id"),
                cst::Expression::from(cst::AbstractionExpression::new(
                    vec![cst::Identifier::new("x")],
                    cst::Expression::from(cst::ApplicationExpression::new(vec![
                        cst::Expression::from(cst::VariableExpression::new(cst::Identifier::new(
                            "x",
                        ))),
                    ])),
                )),
            )),
            cst::Statement::from(cst::ExpressionStatement::new(cst::Expression::from(
                cst::VariableExpression::new(cst::Identifier::new("id")),
            ))),
        ]));
        assert_eq!(expected, result);
    }

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

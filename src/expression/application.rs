use ast;
use expression::Expression;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Application {
    callee: Box<Expression>,
    argument: Box<Expression>,
}

impl Application {
    pub fn new<T, U>(callee: T, argument: U) -> Application
    where
        T: Into<Expression>,
        U: Into<Expression>,
    {
        Application {
            callee: box callee.into(),
            argument: box argument.into(),
        }
    }

    pub fn from_ast<'a>(
        value: &'a ast::ApplicationExpression,
        table: &mut HashMap<&'a str, usize>,
    ) -> Expression {
        let mut iter = value.expressions.iter();
        let callee = iter.next().unwrap();

        if let Some(argument) = iter.next() {
            iter.fold(
                Expression::Application(Application::new(
                    Expression::from_ast(callee, table),
                    Expression::from_ast(argument, table),
                )),
                |callee, argument| {
                    Expression::Application(Application::new(
                        callee,
                        Expression::from_ast(argument, table),
                    ))
                },
            )
        } else {
            Expression::from_ast(callee, table)
        }
    }

    pub fn evaluate1(self) -> Expression {
        match self {
            Application {
                callee: box Expression::Abstraction(callee),
                box argument,
            } => callee.applied(argument),

            Application {
                callee: box Expression::Application(callee),
                box argument,
            } => Expression::Application(Application::new(callee.evaluate1(), argument)),

            _ => Expression::Application(self),
        }
    }

    pub fn shifted(self, d: isize, c: usize) -> Self {
        Application::new(self.callee.shifted(d, c), self.argument.shifted(d, c))
    }

    pub fn substituted(self, j: usize, term: Expression) -> Self {
        let cloned_term = term.clone();
        Application::new(
            self.callee.substituted(j, term),
            self.argument.substituted(j, cloned_term),
        )
    }
}

impl Display for Application {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, r"({} {})", self.callee, self.argument)
    }
}

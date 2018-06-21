mod expression;
mod program;
mod statement;
pub use self::expression::*;
pub use self::program::*;
pub use self::statement::*;

#[derive(Debug, PartialEq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn new<T>(value: T) -> Identifier
    where
        T: Into<String>,
    {
        Identifier(value.into())
    }
}

#[derive(Debug, PartialEq)]
pub struct Number(pub String);

impl Number {
    pub fn new<T>(value: T) -> Number
    where
        T: Into<String>,
    {
        Number(value.into())
    }
}

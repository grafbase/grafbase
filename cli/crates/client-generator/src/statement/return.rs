use std::fmt;

use crate::Expression;

#[derive(Debug)]
pub struct Return {
    expression: Expression,
}

impl Return {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl fmt::Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expression)
    }
}

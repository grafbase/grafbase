use core::fmt;

use super::Expression;

#[derive(Debug)]
pub struct TypeOf {
    expression: Expression,
}

impl TypeOf {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl fmt::Display for TypeOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typeof {}", self.expression)
    }
}

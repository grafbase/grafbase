use core::fmt;

use super::Expression;

pub struct TypeOf<'a> {
    expression: Expression<'a>,
}

impl<'a> TypeOf<'a> {
    pub fn new(expression: impl Into<Expression<'a>>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl<'a> fmt::Display for TypeOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typeof {}", self.expression)
    }
}

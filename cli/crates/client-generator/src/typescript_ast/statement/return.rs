use std::fmt;

use crate::typescript_ast::Expression;

pub struct Return<'a> {
    expression: Expression<'a>,
}

#[allow(dead_code)]
impl<'a> Return<'a> {
    pub fn new(expression: impl Into<Expression<'a>>) -> Self {
        Self {
            expression: expression.into(),
        }
    }
}

impl<'a> fmt::Display for Return<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expression)
    }
}

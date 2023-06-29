use std::fmt;

use super::Expression;

pub struct Equals<'a> {
    left: Expression<'a>,
    right: Expression<'a>,
    strict: bool,
}

#[allow(dead_code)]
impl<'a> Equals<'a> {
    pub fn new(left: impl Into<Expression<'a>>, right: impl Into<Expression<'a>>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
            strict: true,
        }
    }

    #[must_use]
    pub fn non_strict(mut self) -> Self {
        self.strict = false;

        self
    }
}

impl<'a> fmt::Display for Equals<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = if self.strict { "===" } else { "==" };

        write!(f, "{} {} {}", self.left, op, self.right)
    }
}

use std::fmt;

use super::Expression;

#[derive(Debug)]
pub struct Equals {
    left: Expression,
    right: Expression,
    strict: bool,
}

impl Equals {
    pub fn new(left: impl Into<Expression>, right: impl Into<Expression>) -> Self {
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

impl fmt::Display for Equals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = if self.strict { "===" } else { "==" };

        write!(f, "{} {} {}", self.left, op, self.right)
    }
}

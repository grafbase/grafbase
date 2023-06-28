mod assignment;
mod conditional;
mod export;
mod r#return;

use std::fmt;

pub use assignment::Assignment;
pub use conditional::Conditional;
pub use export::Export;
pub use r#return::Return;

pub struct Statement<'a> {
    kind: StatementKind<'a>,
}

impl<'a> fmt::Display for Statement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            StatementKind::Assignment(ref v) => v.fmt(f),
            StatementKind::Conditional(ref v) => v.fmt(f),
            StatementKind::Return(ref v) => v.fmt(f),
        }
    }
}

impl<'a, T> From<T> for Statement<'a>
where
    T: Into<StatementKind<'a>>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

pub enum StatementKind<'a> {
    Assignment(Assignment<'a>),
    Conditional(Conditional<'a>),
    Return(Return<'a>),
}

impl<'a> From<Assignment<'a>> for StatementKind<'a> {
    fn from(value: Assignment<'a>) -> Self {
        Self::Assignment(value)
    }
}

impl<'a> From<Conditional<'a>> for StatementKind<'a> {
    fn from(value: Conditional<'a>) -> Self {
        Self::Conditional(value)
    }
}

impl<'a> From<Return<'a>> for StatementKind<'a> {
    fn from(value: Return<'a>) -> Self {
        Self::Return(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{expect, expect_ts};
    use crate::{expression::Value, statement::Conditional, Block};

    #[test]
    fn single_if() {
        let mut block = Block::new();
        block.push(Value::from(1));

        let conditional = Conditional::new(Value::from(true), block);

        let expected = expect![[r#"
            if (true) {
              1
            }
        "#]];

        expect_ts(&conditional, &expected);
    }

    #[test]
    fn if_else() {
        let mut block = Block::new();
        block.push(Value::from(1));

        let mut conditional = Conditional::new(Value::from(true), block);

        let mut block = Block::new();
        block.push(Value::from(2));

        conditional.r#else(block);

        let expected = expect![[r#"
            if (true) {
              1
            } else {
              2
            }
        "#]];

        expect_ts(&conditional, &expected);
    }

    #[test]
    fn if_else_if_else() {
        let mut block = Block::new();
        block.push(Value::from(1));

        let mut conditional = Conditional::new(Value::from(true), block);

        let mut block = Block::new();
        block.push(Value::from(2));

        conditional.else_if(Value::from(false), block);

        let mut block = Block::new();
        block.push(Value::from(3));

        conditional.r#else(block);

        let expected = expect![[r#"
            if (true) {
              1
            } else if (false) {
              2
            } else {
              3
            }
        "#]];

        expect_ts(&conditional, &expected);
    }
}

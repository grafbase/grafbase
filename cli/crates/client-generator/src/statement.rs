mod assignment;
mod conditional;
mod export;
mod r#return;

use std::fmt;

pub use assignment::Assignment;
pub use conditional::Conditional;
pub use export::Export;
pub use r#return::Return;

#[derive(Debug)]
pub struct Statement {
    kind: StatementKind,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            StatementKind::Assignment(ref v) => v.fmt(f),
            StatementKind::Conditional(ref v) => v.fmt(f),
            StatementKind::Return(ref v) => v.fmt(f),
        }
    }
}

impl<T> From<T> for Statement
where
    T: Into<StatementKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug)]
pub enum StatementKind {
    Assignment(Assignment),
    Conditional(Conditional),
    Return(Return),
}

impl From<Assignment> for StatementKind {
    fn from(value: Assignment) -> Self {
        Self::Assignment(value)
    }
}

impl From<Conditional> for StatementKind {
    fn from(value: Conditional) -> Self {
        Self::Conditional(value)
    }
}

impl From<Return> for StatementKind {
    fn from(value: Return) -> Self {
        Self::Return(value)
    }
}

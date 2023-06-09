use std::fmt::{self, Write};

use crate::{
    expression::{Expression, Value},
    r#type::Type,
    statement::{Assignment, Return, Statement},
    Interface,
};

#[derive(Debug, Default)]
pub struct Block {
    contents: Vec<BlockItem>,
}

impl Block {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, content: impl Into<BlockItem>) {
        self.contents.push(content.into())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\n")?;

        for item in &self.contents {
            writeln!(f, "{item}")?;
        }

        f.write_char('}')?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum BlockItemKind {
    Type(Type),
    Interface(Interface),
    Block(Box<Block>),
    Expression(Expression),
    Statement(Statement),
    Newline,
}

#[derive(Debug)]
pub struct BlockItem {
    kind: BlockItemKind,
}

impl BlockItem {
    #[must_use]
    pub fn newline() -> Self {
        Self {
            kind: BlockItemKind::Newline,
        }
    }
}

impl From<Type> for BlockItemKind {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<Interface> for BlockItemKind {
    fn from(value: Interface) -> Self {
        Self::Interface(value)
    }
}

impl From<Block> for BlockItemKind {
    fn from(value: Block) -> Self {
        Self::Block(Box::new(value))
    }
}

impl From<Statement> for BlockItemKind {
    fn from(value: Statement) -> Self {
        Self::Statement(value)
    }
}

impl From<Expression> for BlockItemKind {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

impl From<Value> for BlockItemKind {
    fn from(value: Value) -> Self {
        Self::Expression(Expression::from(value))
    }
}

impl From<Return> for BlockItemKind {
    fn from(value: Return) -> Self {
        Self::Statement(Statement::from(value))
    }
}

impl From<Assignment> for BlockItemKind {
    fn from(value: Assignment) -> Self {
        Self::Statement(Statement::from(value))
    }
}

impl<T> From<T> for BlockItem
where
    T: Into<BlockItemKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

impl fmt::Display for BlockItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            BlockItemKind::Type(ref t) => t.fmt(f),
            BlockItemKind::Interface(ref i) => i.fmt(f),
            BlockItemKind::Block(ref b) => b.fmt(f),
            BlockItemKind::Expression(ref e) => e.fmt(f),
            BlockItemKind::Statement(ref s) => s.fmt(f),
            BlockItemKind::Newline => writeln!(f),
        }
    }
}

use std::fmt::{self, Write};

use crate::{
    expression::{Expression, Value},
    r#type::Type,
    statement::{Assignment, Return, Statement},
    Interface,
};

#[derive(Default)]
pub struct Block<'a> {
    contents: Vec<BlockItem<'a>>,
}

#[allow(dead_code)]
impl<'a> Block<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, content: impl Into<BlockItem<'a>>) {
        self.contents.push(content.into())
    }
}

impl<'a> fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\n")?;

        for item in &self.contents {
            writeln!(f, "{item}")?;
        }

        f.write_char('}')?;

        Ok(())
    }
}

#[allow(dead_code)]
pub enum BlockItemKind<'a> {
    Type(Type<'a>),
    Interface(Interface<'a>),
    Block(Box<Block<'a>>),
    Expression(Expression<'a>),
    Statement(Statement<'a>),
    Newline,
}

pub struct BlockItem<'a> {
    kind: BlockItemKind<'a>,
}

#[allow(dead_code)]
impl<'a> BlockItem<'a> {
    pub fn new(kind: impl Into<BlockItemKind<'a>>) -> Self {
        Self { kind: kind.into() }
    }

    #[must_use]
    pub fn newline() -> Self {
        Self {
            kind: BlockItemKind::Newline,
        }
    }
}

impl<'a> From<Type<'a>> for BlockItemKind<'a> {
    fn from(value: Type<'a>) -> Self {
        Self::Type(value)
    }
}

impl<'a> From<Interface<'a>> for BlockItemKind<'a> {
    fn from(value: Interface<'a>) -> Self {
        Self::Interface(value)
    }
}

impl<'a> From<Block<'a>> for BlockItemKind<'a> {
    fn from(value: Block<'a>) -> Self {
        Self::Block(Box::new(value))
    }
}

impl<'a> From<Statement<'a>> for BlockItemKind<'a> {
    fn from(value: Statement<'a>) -> Self {
        Self::Statement(value)
    }
}

impl<'a> From<Expression<'a>> for BlockItemKind<'a> {
    fn from(value: Expression<'a>) -> Self {
        Self::Expression(value)
    }
}

impl<'a> From<Value<'a>> for BlockItemKind<'a> {
    fn from(value: Value<'a>) -> Self {
        Self::Expression(Expression::from(value))
    }
}

impl<'a> From<Return<'a>> for BlockItemKind<'a> {
    fn from(value: Return<'a>) -> Self {
        Self::Statement(Statement::from(value))
    }
}

impl<'a> From<Assignment<'a>> for BlockItemKind<'a> {
    fn from(value: Assignment<'a>) -> Self {
        Self::Statement(Statement::from(value))
    }
}

impl<'a, T> From<T> for BlockItem<'a>
where
    T: Into<BlockItemKind<'a>>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

impl<'a> fmt::Display for BlockItem<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{expect, expect_ts};
    use crate::{
        expression::{Object, Value},
        r#type::{Property, StaticType},
        statement::{Assignment, Statement},
        Interface,
    };

    #[test]
    fn basic_block() {
        let mut block = Block::new();

        let mut interface = Interface::new("User");
        interface.push_property(Property::new("id", StaticType::ident("number")));
        interface.push_property(Property::new("name", StaticType::ident("string")));
        block.push(interface);

        block.push(BlockItem::newline());

        let mut object = Object::new();
        object.entry("id", Value::from(1));
        object.entry("name", Value::from("Naukio"));

        let assignment = Assignment::new("myObject", object)
            .r#const()
            .r#type(StaticType::ident("User"));

        block.push(Statement::from(assignment));

        let assignment = Assignment::new("foo", Value::from(1)).r#let();
        block.push(Statement::from(assignment));

        let assignment = Assignment::new("bar", Value::from(1)).var();
        block.push(Statement::from(assignment));

        let assignment = Assignment::new("bar", Value::from(2));
        block.push(Statement::from(assignment));

        let expected = expect![[r#"
            {
              interface User {
                id: number
                name: string
              }

              const myObject: User = { id: 1, name: 'Naukio' }
              let foo = 1
              var bar = 1
              bar = 2
            }
        "#]];

        expect_ts(&block, &expected);
    }
}

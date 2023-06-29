use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use super::{Block, Property, PropertyValue, TypeKind};

pub struct Function<'a> {
    inner: FunctionBody<'a>,
}

#[allow(dead_code)]
impl<'a> Function<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, body: Block<'a>) -> Self {
        let inner = FunctionBody {
            name: name.into(),
            params: Vec::new(),
            returns: None,
            body,
        };

        Self { inner }
    }

    pub fn returns(mut self, r#type: impl Into<TypeKind<'a>>) -> Self {
        self.inner.returns = Some(r#type.into());
        self
    }

    pub fn push_param(mut self, key: impl Into<Cow<'a, str>>, value: impl Into<PropertyValue<'a>>) -> Self {
        self.inner.params.push(Property::new(key, value));
        self
    }
}

impl<'a> fmt::Display for Function<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function {}", self.inner)
    }
}

pub struct FunctionBody<'a> {
    pub name: Cow<'a, str>,
    pub params: Vec<Property<'a>>,
    pub returns: Option<TypeKind<'a>>,
    pub body: Block<'a>,
}

impl<'a> fmt::Display for FunctionBody<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;

        for param in &self.params {
            write!(f, "{param},")?;
        }

        f.write_char(')')?;

        if let Some(ref returns) = self.returns {
            write!(f, ": {returns} {}", self.body)?;
        } else {
            write!(f, " {}", self.body)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_helpers::{expect, expect_ts},
        typescript_ast::{Block, Export, Identifier, Return, StaticType},
    };

    use super::Function;

    #[test]
    fn basic_function() {
        let mut block = Block::new();
        block.push(Return::new(Identifier::new("foo")));

        let function = Function::new("bar", block)
            .push_param("foo", StaticType::ident("string"))
            .returns(StaticType::ident("string"));

        let expected = expect![[r#"
            function bar(foo: string): string {
              return foo
            }
        "#]];

        expect_ts(&function, &expected);
    }

    #[test]
    fn export_function() {
        let mut block = Block::new();
        block.push(Return::new(Identifier::new("foo")));

        let function = Function::new("bar", block)
            .push_param("foo", StaticType::ident("string"))
            .returns(StaticType::ident("string"));

        let expected = expect![[r#"
            export function bar(foo: string): string {
              return foo
            }
        "#]];

        expect_ts(&Export::new(function), &expected);
    }
}

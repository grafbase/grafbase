use std::borrow::Cow;
use std::fmt;

use crate::common::Identifier;
use crate::r#type::TypeKind;
use crate::Expression;

pub struct Assignment<'a> {
    left: Identifier<'a>,
    right: Expression<'a>,
    mutability: Mutability,
    r#type: Option<TypeKind<'a>>,
}

#[allow(dead_code)]
impl<'a> Assignment<'a> {
    pub fn new(left: impl Into<Cow<'a, str>>, right: impl Into<Expression<'a>>) -> Self {
        Self {
            left: Identifier::new(left),
            right: right.into(),
            mutability: Mutability::Existing,
            r#type: None,
        }
    }

    #[must_use]
    pub fn r#const(mut self) -> Self {
        self.mutability = Mutability::Const;
        self
    }

    #[must_use]
    pub fn var(mut self) -> Self {
        self.mutability = Mutability::Var;
        self
    }

    #[must_use]
    pub fn r#let(mut self) -> Self {
        self.mutability = Mutability::Let;
        self
    }

    pub fn r#type(mut self, typedef: impl Into<TypeKind<'a>>) -> Self {
        self.r#type = Some(typedef.into());
        self
    }
}

impl<'a> fmt::Display for Assignment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.mutability, self.left)?;

        if let Some(ref typedef) = self.r#type {
            write!(f, ": {typedef}")?;
        }

        write!(f, " = {}", self.right)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Existing,
    Const,
    Var,
    Let,
}

impl fmt::Display for Mutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Existing => Ok(()),
            Self::Const => f.write_str("const "),
            Self::Var => f.write_str("var "),
            Self::Let => f.write_str("let "),
        }
    }
}

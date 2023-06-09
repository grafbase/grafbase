use std::borrow::Cow;
use std::fmt;

use crate::common::Identifier;
use crate::r#type::TypeKind;
use crate::Expression;

#[derive(Debug)]
pub struct Assignment {
    left: Identifier,
    right: Expression,
    mutability: Mutability,
    r#type: Option<TypeKind>,
}

impl Assignment {
    pub fn new(left: impl Into<Cow<'static, str>>, right: impl Into<Expression>) -> Self {
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

    pub fn r#type(mut self, typedef: impl Into<TypeKind>) -> Self {
        self.r#type = Some(typedef.into());
        self
    }
}

impl fmt::Display for Assignment {
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

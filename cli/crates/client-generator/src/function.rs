use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use crate::{
    r#type::{Property, PropertyValue, TypeKind},
    Block,
};

#[derive(Debug)]
pub struct Function {
    inner: FunctionBody,
}

impl Function {
    pub fn new(name: impl Into<Cow<'static, str>>, body: Block) -> Self {
        let inner = FunctionBody {
            name: name.into(),
            params: Vec::new(),
            returns: None,
            body,
        };

        Self { inner }
    }

    pub fn returns(mut self, r#type: impl Into<TypeKind>) -> Self {
        self.inner.returns = Some(r#type.into());
        self
    }

    pub fn push_param(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) -> Self {
        self.inner.params.push(Property::new(key, value));
        self
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function {}", self.inner)
    }
}

#[derive(Debug)]
pub struct FunctionBody {
    pub name: Cow<'static, str>,
    pub params: Vec<Property>,
    pub returns: Option<TypeKind>,
    pub body: Block,
}

impl fmt::Display for FunctionBody {
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

use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use crate::common::{Identifier, Quoted};

use super::{StaticType, TypeName};

#[derive(Debug, Clone)]
pub struct TypeIdentifier {
    name: TypeName,
    params: Vec<StaticType>,
    extends: Option<Box<StaticType>>,
}

impl TypeIdentifier {
    pub fn ident(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeName::Ident(Identifier::new(name)),
            params: Vec::new(),
            extends: None,
        }
    }

    pub fn string(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: TypeName::String(Quoted::new(name)),
            params: Vec::new(),
            extends: None,
        }
    }

    #[must_use]
    pub fn extends(mut self, ident: StaticType) -> Self {
        self.extends = Some(Box::new(ident));

        self
    }

    pub fn push_param(&mut self, param: StaticType) {
        self.params.push(param);
    }
}

impl fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)?;

        if !self.params.is_empty() {
            f.write_char('<')?;

            for (i, param) in self.params.iter().enumerate() {
                param.fmt(f)?;

                if i < self.params.len() - 1 {
                    f.write_str(", ")?;
                }
            }

            f.write_char('>')?;
        }

        if let Some(ref extends) = self.extends {
            write!(f, " extends {extends}")?;
        }

        Ok(())
    }
}

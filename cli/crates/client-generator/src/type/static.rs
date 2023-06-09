use std::{borrow::Cow, fmt};

use super::{TypeCondition, TypeIdentifier};

#[derive(Debug, Clone)]
pub struct StaticType {
    identifier: TypeIdentifier,
    or: Vec<StaticType>,
    condition: Option<Box<TypeCondition>>,
    keyof: bool,
}

impl StaticType {
    pub fn ident(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: TypeIdentifier::ident(name),
            or: Vec::new(),
            condition: None,
            keyof: false,
        }
    }

    pub fn string(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: TypeIdentifier::string(name),
            or: Vec::new(),
            condition: None,
            keyof: false,
        }
    }

    #[must_use]
    pub fn extends(mut self, ident: StaticType) -> Self {
        self.identifier = self.identifier.extends(ident);
        self
    }

    #[must_use]
    pub fn or(mut self, ident: StaticType) -> Self {
        self.or.push(ident);
        self
    }

    #[must_use]
    pub fn condition(mut self, condition: TypeCondition) -> Self {
        self.condition = Some(Box::new(condition));
        self
    }

    #[must_use]
    pub fn keyof(mut self) -> Self {
        self.keyof = true;
        self
    }

    pub fn push_param(&mut self, param: StaticType) {
        self.identifier.push_param(param);
    }
}

impl fmt::Display for StaticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.keyof {
            f.write_str("keyof ")?;
        }

        self.identifier.fmt(f)?;

        if !self.or.is_empty() {
            f.write_str(" | ")?;

            for (i, ident) in self.or.iter().enumerate() {
                ident.fmt(f)?;

                if i < self.or.len() - 1 {
                    f.write_str(" | ")?;
                }
            }
        }

        if let Some(ref condition) = self.condition {
            write!(f, " {condition}")?;
        }

        Ok(())
    }
}

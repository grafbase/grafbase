use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use async_graphql_parser::types::BaseType;

use crate::common::{Identifier, Quoted};

use super::{StaticType, TypeName};

#[derive(Clone)]
pub struct TypeIdentifier<'a> {
    name: TypeName<'a>,
    params: Vec<StaticType<'a>>,
    extends: Option<Box<StaticType<'a>>>,
    array: bool,
}

impl<'a> TypeIdentifier<'a> {
    pub fn ident(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: TypeName::Ident(Identifier::new(name)),
            params: Vec::new(),
            extends: None,
            array: false,
        }
    }

    pub fn string(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: TypeName::String(Quoted::new(name)),
            params: Vec::new(),
            extends: None,
            array: false,
        }
    }

    pub fn extends(&mut self, ident: StaticType<'a>) {
        self.extends = Some(Box::new(ident));
    }

    pub fn array(&mut self) {
        self.array = true;
    }

    pub fn from_graphql(base: &'a BaseType) -> Self {
        match base {
            BaseType::Named(ref name) if name.as_str() == "String" => Self::ident("string"),
            BaseType::Named(ref name) if name.as_str() == "ID" => Self::ident("string"),
            BaseType::Named(ref name) if name.as_str() == "Int" => Self::ident("number"),
            BaseType::Named(ref name) if name.as_str() == "Float" => Self::ident("number"),
            BaseType::Named(ref name) if name.as_str() == "Boolean" => Self::ident("boolean"),
            BaseType::Named(ref name) if name.as_str() == "Date" => Self::ident("Date"),
            BaseType::Named(ref name) if name.as_str() == "DateTime" => Self::ident("Date"),
            BaseType::Named(ref name) if name.as_str() == "Timestamp" => Self::ident("Date"),
            BaseType::Named(ref name) if name.as_str() == "Email" => Self::ident("string"),
            BaseType::Named(ref name) if name.as_str() == "IPAddress" => Self::ident("string"),
            BaseType::Named(ref name) if name.as_str() == "URL" => Self::ident("string"),
            BaseType::Named(ref name) if name.as_str() == "JSON" => Self::ident("object"),
            BaseType::Named(ref name) if name.as_str() == "PhoneNumber" => Self::ident("string"),
            BaseType::Named(ref name) => Self::ident(name.as_str()),
            BaseType::List(ref base) => {
                let mut identifier = Self::from_graphql(&base.base);
                identifier.array();
                identifier
            }
        }
    }

    pub fn push_param(&mut self, param: StaticType<'a>) {
        self.params.push(param);
    }
}

impl<'a> fmt::Display for TypeIdentifier<'a> {
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

        if self.array {
            f.write_str("[]")?;
        }

        if let Some(ref extends) = self.extends {
            write!(f, " extends {extends}")?;
        }

        Ok(())
    }
}

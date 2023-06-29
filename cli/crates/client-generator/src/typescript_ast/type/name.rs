use core::fmt;

use crate::typescript_ast::{Identifier, Quoted};

#[derive(Debug, Clone)]
pub enum TypeName<'a> {
    Ident(Identifier<'a>),
    String(Quoted<'a>),
}

impl<'a> fmt::Display for TypeName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeName::Ident(ref i) => i.fmt(f),
            TypeName::String(ref i) => i.fmt(f),
        }
    }
}

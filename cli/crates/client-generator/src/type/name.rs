use core::fmt;

use crate::common::{Identifier, Quoted};

#[derive(Debug, Clone)]
pub enum TypeName {
    Ident(Identifier),
    String(Quoted),
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeName::Ident(ref i) => i.fmt(f),
            TypeName::String(ref i) => i.fmt(f),
        }
    }
}

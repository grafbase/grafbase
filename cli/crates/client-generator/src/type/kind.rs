use std::fmt;

use super::{MappedType, StaticType};

#[derive(Clone, Debug)]
pub enum TypeKind<'a> {
    Static(StaticType<'a>),
    Mapped(MappedType<'a>),
}

impl<'a> From<StaticType<'a>> for TypeKind<'a> {
    fn from(value: StaticType<'a>) -> Self {
        Self::Static(value)
    }
}

impl<'a> From<MappedType<'a>> for TypeKind<'a> {
    fn from(value: MappedType<'a>) -> Self {
        Self::Mapped(value)
    }
}

impl<'a> fmt::Display for TypeKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Static(s) => s.fmt(f),
            TypeKind::Mapped(m) => m.fmt(f),
        }
    }
}

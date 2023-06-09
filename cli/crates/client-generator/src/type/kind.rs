use std::fmt;

use super::{MappedType, StaticType};

#[derive(Debug, Clone)]
pub enum TypeKind {
    Static(StaticType),
    Mapped(MappedType),
}

impl From<StaticType> for TypeKind {
    fn from(value: StaticType) -> Self {
        Self::Static(value)
    }
}

impl From<MappedType> for TypeKind {
    fn from(value: MappedType) -> Self {
        Self::Mapped(value)
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Static(s) => s.fmt(f),
            TypeKind::Mapped(m) => m.fmt(f),
        }
    }
}

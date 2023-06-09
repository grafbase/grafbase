use std::fmt;

use super::{Property, TypeGenerator, TypeKind};

#[derive(Debug, Clone)]
pub struct MappedType {
    source: TypeMapSource,
    definition: Box<TypeKind>,
}

impl MappedType {
    pub fn new(source: impl Into<TypeMapSource>, definition: impl Into<TypeKind>) -> Self {
        Self {
            source: source.into(),
            definition: Box::new(definition.into()),
        }
    }
}

impl fmt::Display for MappedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ [{}]: {} }}", self.source, self.definition)
    }
}

#[derive(Debug, Clone)]
pub enum TypeMapSource {
    Generator(TypeGenerator),
    Static(Property),
}

impl From<TypeGenerator> for TypeMapSource {
    fn from(value: TypeGenerator) -> Self {
        Self::Generator(value)
    }
}

impl From<Property> for TypeMapSource {
    fn from(value: Property) -> Self {
        Self::Static(value)
    }
}

impl fmt::Display for TypeMapSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeMapSource::Generator(g) => g.fmt(f),
            TypeMapSource::Static(s) => s.fmt(f),
        }
    }
}

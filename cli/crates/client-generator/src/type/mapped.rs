use std::fmt;

use super::{Property, TypeGenerator, TypeKind};

#[derive(Clone, Debug)]
pub struct MappedType<'a> {
    source: TypeMapSource<'a>,
    definition: Box<TypeKind<'a>>,
}

#[allow(dead_code)]
impl<'a> MappedType<'a> {
    pub fn new(source: impl Into<TypeMapSource<'a>>, definition: impl Into<TypeKind<'a>>) -> Self {
        Self {
            source: source.into(),
            definition: Box::new(definition.into()),
        }
    }
}

impl<'a> fmt::Display for MappedType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ [{}]: {} }}", self.source, self.definition)
    }
}

#[derive(Clone, Debug)]
pub enum TypeMapSource<'a> {
    Generator(TypeGenerator<'a>),
    Static(Property<'a>),
}

impl<'a> From<TypeGenerator<'a>> for TypeMapSource<'a> {
    fn from(value: TypeGenerator<'a>) -> Self {
        Self::Generator(value)
    }
}

impl<'a> From<Property<'a>> for TypeMapSource<'a> {
    fn from(value: Property<'a>) -> Self {
        Self::Static(value)
    }
}

impl<'a> fmt::Display for TypeMapSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeMapSource::Generator(g) => g.fmt(f),
            TypeMapSource::Static(s) => s.fmt(f),
        }
    }
}

//! Utilities for correctly constructing graphql types from names

use engine::registry::{InputValueType, MetaFieldType, NamedType};

/// An extension trait for `engine::registry::TypeName`
///
/// Provides convenience functions for converting into `MetaFieldType`
/// (and eventually a `MetaInputType`)
#[allow(clippy::wrong_self_convention)] // We only impl on a reference, so the names are fine
pub trait TypeNameExt<'a> {
    fn as_nullable(self) -> TypeBuilder<'a>;
    fn as_non_null(self) -> TypeBuilder<'a>;
}

impl<'a> TypeNameExt<'a> for &'a NamedType<'_> {
    fn as_nullable(self) -> TypeBuilder<'a> {
        TypeBuilder {
            name: self.borrow(),
            wrapping: vec![],
        }
    }

    fn as_non_null(self) -> TypeBuilder<'a> {
        TypeBuilder {
            name: self.borrow(),
            wrapping: vec![WrappingType::NonNull],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeBuilder<'a> {
    name: NamedType<'a>,
    wrapping: Vec<WrappingType>,
}

impl<'a> TypeBuilder<'a> {
    pub fn list(mut self) -> TypeBuilder<'a> {
        self.wrapping.push(WrappingType::List);
        self
    }

    pub fn nullable(mut self) -> TypeBuilder<'a> {
        if let Some(WrappingType::NonNull) = self.wrapping.last() {
            self.wrapping.pop();
        }
        self
    }

    pub fn non_null(mut self) -> TypeBuilder<'a> {
        if !matches!(self.wrapping.last(), Some(WrappingType::NonNull)) {
            self.wrapping.push(WrappingType::NonNull);
        }
        self
    }
}

impl From<TypeBuilder<'_>> for MetaFieldType {
    fn from(value: TypeBuilder<'_>) -> Self {
        value.to_string().into()
    }
}

impl From<TypeBuilder<'_>> for InputValueType {
    fn from(value: TypeBuilder<'_>) -> Self {
        value.to_string().into()
    }
}

impl std::fmt::Display for TypeBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TypeBuilder { name, wrapping } = self;
        for wrapping_type in wrapping {
            match wrapping_type {
                WrappingType::List => write!(f, "[")?,
                WrappingType::NonNull => {}
            }
        }
        write!(f, "{name}")?;
        for wrapping_type in wrapping.iter().rev() {
            match wrapping_type {
                WrappingType::List => write!(f, "]")?,
                WrappingType::NonNull => write!(f, "!")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum WrappingType {
    NonNull,
    List,
}

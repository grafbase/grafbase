//! Various types for working with GraphQL type names

use std::borrow::{Borrow, Cow};

use engine_value::Name;
use registry_v1::{InputValueType, MetaFieldType};

use super::{
    type_kinds::{InputType, OutputType, SelectionSetTarget},
    MetaType, ObjectType,
};

/// A trait for types that represent type names in someway.
///
/// This is used by the lookup function on the `Registry` to provide a bit of convenience
/// and type-safety around retrieving types from the registry.
pub trait TypeReference {
    /// The kind of type we expect this `TypeName` to represent in the `Registry`.
    type ExpectedType<'a>;

    /// The name of the type
    fn named_type(&self) -> NamedType<'_>;

    fn lookup_meta<'a>(&self, registry: &'a registry_v2::Registry) -> Option<registry_v2::MetaType<'a>> {
        registry.lookup_type(self.named_type().as_str())
    }
}

/// Defines basic string conversion functionality for a string wrapper.
///
/// We've a lot of them in this file, so this is handy.
macro_rules! def_string_conversions {
    ($ty:ident) => {
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $ty {
            #[allow(dead_code)]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        // Might be good to get rid of these two _eventually_ but for now
        // we've got a lot of old stuff that still works directly with strings
        impl From<&str> for $ty {
            fn from(value: &str) -> $ty {
                $ty(value.to_string())
            }
        }

        impl From<String> for $ty {
            fn from(value: String) -> $ty {
                $ty(value)
            }
        }
    };
}

impl TypeReference for registry_v2::MetaFieldType<'_> {
    type ExpectedType<'a> = OutputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Owned(self.to_string()))
    }

    fn lookup_meta<'a>(&self, registry: &'a registry_v2::Registry) -> Option<registry_v2::MetaType<'a>> {
        // This is annoyingly laborious because lifetimes.  Can be simplified once regisry v1 is gone
        Some(registry.read(self.id()).named_type())
    }
}

impl TypeReference for registry_v2::MetaInputValueType<'_> {
    type ExpectedType<'a> = InputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Owned(self.to_string()))
    }

    fn lookup_meta<'a>(&self, registry: &'a registry_v2::Registry) -> Option<registry_v2::MetaType<'a>> {
        // This is annoyingly laborious because lifetimes.  Can be simplified once regisry v1 is gone
        Some(registry.read(self.id()).named_type())
    }
}

impl TypeReference for MetaFieldType {
    type ExpectedType<'a> = OutputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(named_type_from_type_str(self.as_str())))
    }
}

impl TypeReference for InputValueType {
    type ExpectedType<'a> = InputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(named_type_from_type_str(self.as_str())))
    }
}

/// The name of a Grafbase Model type, with no wrapping types
///
/// Currently this is always going to be an Object, but at some point it might be extended
/// to support others.
#[derive(Clone, Default, Hash, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ModelName(String);

def_string_conversions!(ModelName);

impl TypeReference for ModelName {
    type ExpectedType<'a> = &'a ObjectType;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(&self.0))
    }

    fn lookup_meta<'a>(&self, registry: &'a registry_v2::Registry) -> Option<registry_v2::MetaType<'a>> {
        registry.lookup_type(&self.0)
    }
}

/// A type condition from an inline fragment spread or a fragment definition.
#[derive(Clone, Debug)]
pub struct TypeCondition(String);

def_string_conversions!(TypeCondition);

impl TypeReference for TypeCondition {
    type ExpectedType<'a> = SelectionSetTarget<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(&self.0))
    }
}

/// A named GraphQL type without any non-null or list wrappers
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct NamedType<'a>(Cow<'a, str>);

impl NamedType<'_> {
    pub fn borrow(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(self.0.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_primitive_type(&self) -> bool {
        matches!(self.0.borrow(), "String" | "Float" | "Boolean" | "ID" | "Int")
    }
}

impl TypeReference for NamedType<'_> {
    type ExpectedType<'a> = &'a MetaType;

    fn named_type(&self) -> NamedType<'_> {
        self.clone()
    }
}

impl From<String> for NamedType<'static> {
    fn from(value: String) -> Self {
        NamedType(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for NamedType<'a> {
    fn from(value: &'a str) -> Self {
        NamedType(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a String> for NamedType<'a> {
    fn from(value: &'a String) -> Self {
        NamedType(Cow::Borrowed(value.as_str()))
    }
}

impl std::fmt::Display for NamedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a Name> for NamedType<'a> {
    fn from(value: &'a Name) -> Self {
        NamedType(Cow::Borrowed(value.as_str()))
    }
}

/// Strips the NonNull and List wrappers from a type string to get the
/// named type within.
fn named_type_from_type_str(meta: &str) -> &str {
    let mut nested = Some(meta);

    if meta.starts_with('[') && meta.ends_with(']') {
        nested = nested.and_then(|x| x.strip_prefix('['));
        nested = nested.and_then(|x| x.strip_suffix(']'));
        return named_type_from_type_str(nested.expect("Can't fail"));
    }

    if meta.ends_with('!') {
        nested = nested.and_then(|x| x.strip_suffix('!'));
        return named_type_from_type_str(nested.expect("Can't fail"));
    }

    nested.expect("Can't fail")
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WrappingType {
    NonNull,
    List,
}

impl WrappingType {
    pub fn all_for(ty: &str) -> Vec<WrappingType> {
        WrappingTypeIter(ty.chars()).collect()
    }
}

pub struct WrappingTypeIter<'a>(std::str::Chars<'a>);

impl Iterator for WrappingTypeIter<'_> {
    type Item = WrappingType;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next_back()? {
            '!' => Some(WrappingType::NonNull),
            ']' => Some(WrappingType::List),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping_type_iter() {
        let wrapping_types = |s: &str| WrappingTypeIter(s.chars()).collect::<Vec<_>>();
        assert_eq!(wrapping_types("String"), vec![]);
        assert_eq!(wrapping_types("String!"), vec![WrappingType::NonNull]);
        assert_eq!(
            wrapping_types("[String]!"),
            vec![WrappingType::NonNull, WrappingType::List]
        );
        assert_eq!(wrapping_types("[String]"), vec![WrappingType::List]);
        assert_eq!(
            wrapping_types("[String!]"),
            vec![WrappingType::List, WrappingType::NonNull]
        );
        assert_eq!(
            wrapping_types("[String!]!"),
            vec![WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
        );
        assert_eq!(
            wrapping_types("[[String!]]"),
            vec![WrappingType::List, WrappingType::List, WrappingType::NonNull]
        );
    }
}

//! Various types for working with GraphQL type names

use std::borrow::{Borrow, Cow};

use engine_value::Name;

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
}

impl TypeReference for Name {
    type ExpectedType<'a> = &'a MetaType;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(self.as_str()))
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

/// The type of a MetaField
///
/// This is just a newtype around a string in SDL type notation (e.g. `[Int]!`).
///
/// Using a newtype allows us to enforce a bit of type safety, implement methods
/// on the type etc. etc.
#[derive(Clone, Default, Hash, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct MetaFieldType(String);

def_string_conversions!(MetaFieldType);

impl MetaFieldType {
    pub fn is_non_null(&self) -> bool {
        // This makes me sad, but for now lets live with it
        self.0.ends_with('!')
    }

    pub fn is_nullable(&self) -> bool {
        // This makes me sad, but for now lets live with it
        !self.0.ends_with('!')
    }

    pub fn is_list(&self) -> bool {
        // Note that we do starts_with here to include both nullable and non-nullable
        // lists.
        self.0.starts_with('[')
    }

    pub fn base_type_name(&self) -> &str {
        named_type_from_type_str(&self.0)
    }

    pub fn wrapping_types(&self) -> WrappingTypeIter<'_> {
        WrappingTypeIter(self.as_str().chars())
    }
}

impl TypeReference for MetaFieldType {
    type ExpectedType<'a> = OutputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(named_type_from_type_str(&self.0)))
    }
}

/// The type of a MetaInputValue
///
/// This is just a newtype around a string in SDL type notation (e.g. `[Int]!`).
///
/// Using a newtype allows us to enforce a bit of type safety, implement methods
/// on the type etc. etc.
#[derive(Clone, Default, Hash, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct InputValueType(String);

def_string_conversions!(InputValueType);

impl InputValueType {
    pub fn is_non_null(&self) -> bool {
        // This makes me sad, but for now lets live with it
        self.0.ends_with('!')
    }

    pub fn is_list(&self) -> bool {
        // Note that we do starts_with here to include both nullable and non-nullable
        // lists.
        self.0.starts_with('[')
    }
}

impl TypeReference for InputValueType {
    type ExpectedType<'a> = InputType<'a>;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(named_type_from_type_str(&self.0)))
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

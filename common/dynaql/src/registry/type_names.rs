//! Various types for working with GraphQL type names

use std::borrow::{Borrow, Cow};

use super::{MetaType, ObjectType};

/// A trait for types that represent type names in someway.
///
/// This is used by the lookup function on the `Registry` to provide a bit of convenience
/// and type-safety around retrieving types from the registry.
pub trait TypeReference {
    /// The kind of type we expect this `TypeName` to represent in the `Registry`.
    type ExpectedType;

    /// The name of the type
    fn named_type(&self) -> NamedType<'_>;
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
    pub fn as_str(&self) -> &str {
        &self.0
    }

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

impl TypeReference for MetaFieldType {
    // TODO: make an OutputType enum and use it here
    type ExpectedType = MetaType;

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

impl ModelName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TypeReference for ModelName {
    type ExpectedType = ObjectType;

    fn named_type(&self) -> NamedType<'_> {
        NamedType(Cow::Borrowed(&self.0))
    }
}

/// A named GraphQL type without any non-null or list wrappers
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedType<'a>(Cow<'a, str>);

impl NamedType<'_> {
    pub fn borrow<'a>(&'a self) -> NamedType<'a> {
        NamedType(Cow::Borrowed(self.0.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_primitive_type(&self) -> bool {
        matches!(
            self.0.borrow(),
            "String" | "Float" | "Boolean" | "ID" | "Int"
        )
    }
}

impl TypeReference for NamedType<'_> {
    type ExpectedType = MetaType;

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

impl std::fmt::Display for NamedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

use crate::registry_impl::named_type_from_type_str;

/// The type of a MetaField
///
/// This is just a newtype around a string in SDL type notation (e.g. `[Int]!`).
///
/// Using a newtype allows us to enforce a bit of type safety, implement methods
/// on the type etc. etc.
#[derive(Clone, Default, Hash, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct MetaFieldType(pub(crate) String);

/// The type of a MetaInputValue
///
/// This is just a newtype around a string in SDL type notation (e.g. `[Int]!`).
///
/// Using a newtype allows us to enforce a bit of type safety, implement methods
/// on the type etc. etc.
#[derive(Clone, Default, Hash, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct InputValueType(pub(crate) String);

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

    // pub fn wrapping_types(&self) -> WrappingTypeIter<'_> {
    //     WrappingTypeIter(self.as_str().chars())
    // }
}

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

    pub fn base_type_name(&self) -> &str {
        named_type_from_type_str(&self.0)
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

def_string_conversions!(MetaFieldType);
def_string_conversions!(InputValueType);

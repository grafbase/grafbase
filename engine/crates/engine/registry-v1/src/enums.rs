use indexmap::IndexMap;
use registry_v2::Deprecation;

use crate::MetaType;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct EnumType {
    pub name: String,
    pub description: Option<String>,
    pub enum_values: IndexMap<String, MetaEnumValue>,
    pub rust_typename: String,
}

impl EnumType {
    pub fn new(name: String, values: impl IntoIterator<Item = MetaEnumValue>) -> Self {
        EnumType {
            rust_typename: name.clone(),
            name,
            enum_values: values.into_iter().map(|value| (value.name.clone(), value)).collect(),
            description: None,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        EnumType { description, ..self }
    }
}

impl From<EnumType> for MetaType {
    fn from(val: EnumType) -> Self {
        MetaType::Enum(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Deprecation)]
#[derive(Clone, Debug, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetaEnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecation: Deprecation,
    // The value that will be used for this MetaEnumValue when sent to a
    // non-GraphQL downstream API
    pub value: Option<String>,
}

impl MetaEnumValue {
    pub fn new(name: String) -> Self {
        MetaEnumValue {
            name,
            description: None,
            deprecation: Deprecation::NoDeprecated,
            value: None,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        MetaEnumValue { description, ..self }
    }

    pub fn with_deprecation(self, deprecation: Deprecation) -> Self {
        MetaEnumValue { deprecation, ..self }
    }
}

impl Eq for MetaEnumValue {}

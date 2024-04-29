use engine_value::ConstValue;
use registry_v2::validators::DynValidator;

use crate::{field_types::InputValueType, serde_preserve_enum};

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, bool)]
#[derive(derivative::Derivative, Clone, serde::Deserialize, serde::Serialize)]
#[derivative(Debug, Hash, PartialEq)]
pub struct MetaInputValue {
    pub name: String,
    pub description: Option<String>,
    pub ty: InputValueType,
    #[derivative(Hash = "ignore")]
    #[serde(with = "serde_preserve_enum")]
    pub default_value: Option<engine_value::ConstValue>,
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub validators: Option<Vec<DynValidator>>,
    pub is_secret: bool,
    pub rename: Option<String>,
}

impl MetaInputValue {
    pub fn new(name: impl Into<String>, ty: impl Into<InputValueType>) -> MetaInputValue {
        MetaInputValue {
            name: name.into(),
            description: None,
            ty: ty.into(),
            default_value: None,
            validators: None,
            is_secret: false,
            rename: None,
        }
    }

    pub fn with_description(self, description: impl Into<String>) -> MetaInputValue {
        MetaInputValue {
            description: Some(description.into()),
            ..self
        }
    }

    pub fn with_rename(self, rename: Option<String>) -> MetaInputValue {
        MetaInputValue { rename, ..self }
    }

    pub fn with_default(self, default: ConstValue) -> MetaInputValue {
        MetaInputValue {
            default_value: Some(default),
            ..self
        }
    }
}

impl Eq for MetaInputValue {}

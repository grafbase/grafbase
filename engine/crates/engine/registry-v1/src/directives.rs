use indexmap::IndexMap;
use registry_v2::DirectiveLocation;

use crate::MetaInputValue;

#[derive(Clone, derivative::Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct MetaDirective {
    pub name: String,
    pub description: Option<String>,
    pub locations: Vec<DirectiveLocation>,
    pub args: IndexMap<String, MetaInputValue>,
    pub is_repeatable: bool,
}

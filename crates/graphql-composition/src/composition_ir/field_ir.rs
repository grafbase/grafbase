use super::*;

#[derive(Clone)]
pub(crate) struct FieldIr {
    pub(crate) parent_definition_name: federated::StringId,
    pub(crate) field_name: federated::StringId,
    pub(crate) field_type: subgraphs::FieldType,
    pub(crate) arguments: federated::InputValueDefinitions,

    pub(crate) directives: Vec<Directive>,

    pub(crate) description: Option<federated::StringId>,
}

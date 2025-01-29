use super::*;

pub(crate) struct DirectiveDefinitionIr {
    pub(crate) name: federated::StringId,
    pub(crate) locations: federated::DirectiveLocations,
    pub(crate) arguments: Vec<InputValueDefinitionIr>,
    pub(crate) repeatable: bool,
}

use super::*;

pub(crate) struct DirectiveDefinitionIr {
    pub(crate) name: federated::StringId,
    pub(crate) locations: federated::DirectiveLocations,
    pub(crate) arguments: federated::InputValueDefinitions,
    pub(crate) repeatable: bool,
}

use crate::{StringId, SubgraphId, Value};

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct ExtensionDirective {
    // FIXME: To be replaced with an internal id, like subgraph_id
    pub extension_id: extension::Id,
    pub subgraph_id: SubgraphId,
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

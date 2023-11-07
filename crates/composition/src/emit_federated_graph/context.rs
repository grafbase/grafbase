use super::field_types_map::FieldTypesMap;
use crate::{subgraphs, VecExt};
use grafbase_federated_graph as federated;
use std::collections::HashMap;

pub(super) struct Context<'a> {
    pub(super) field_types_map: &'a mut FieldTypesMap,
    pub(super) out: &'a mut federated::FederatedGraph,
    pub(super) subgraphs: &'a subgraphs::Subgraphs,
    pub(super) definitions: HashMap<subgraphs::StringId, federated::Definition>,
    pub(super) strings_map: HashMap<subgraphs::StringId, federated::StringId>,
}

impl Context<'_> {
    /// Subgraphs string -> federated graph string.
    pub(crate) fn insert_string(
        &mut self,
        string: subgraphs::StringWalker<'_>,
    ) -> federated::StringId {
        *self.strings_map.entry(string.id).or_insert_with(|| {
            federated::StringId(self.out.strings.push_return_idx(string.as_str().to_owned()))
        })
    }
}

use std::collections::BTreeSet;

use super::DefinitionId;
use crate::Subgraphs;

/// All the unions in all subgraphs.
#[derive(Default, Debug)]
pub(crate) struct Unions(
    /// (union, member)
    BTreeSet<(DefinitionId, DefinitionId)>,
);

impl Subgraphs {
    pub(crate) fn iter_union_members(&self, union_id: DefinitionId) -> impl Iterator<Item = DefinitionId> + '_ {
        self.unions
            .0
            .range((union_id, DefinitionId(usize::MIN))..(union_id, DefinitionId(usize::MAX)))
            .map(|(_, member)| *member)
    }

    pub(crate) fn push_union_member(&mut self, union: DefinitionId, member: DefinitionId) {
        self.unions.0.insert((union, member));
    }
}

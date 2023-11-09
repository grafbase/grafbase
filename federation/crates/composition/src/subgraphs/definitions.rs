use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DefinitionId(pub(super) usize);

// Invariant: `definitions` is sorted by `Definition::subgraph_id`. We rely on it for binary search.
#[derive(Default)]
pub(crate) struct Definitions(Vec<Definition>);

pub(crate) struct Definition {
    subgraph_id: SubgraphId,
    name: StringId,
    kind: DefinitionKind,
    pub(crate) is_shareable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DefinitionKind {
    Object,
    Interface,
    Union,
    InputObject,
    Scalar,
    Enum,
}

impl Subgraphs {
    pub(crate) fn definition_by_name(
        &mut self,
        name: &str,
        subgraph_id: SubgraphId,
    ) -> DefinitionId {
        let interned_name = self.strings.intern(name);
        self.definition_names[&(interned_name, subgraph_id)]
    }
    pub(crate) fn set_shareable(&mut self, definition_id: DefinitionId) {
        self.definitions.0[definition_id.0].is_shareable = true;
    }

    pub(crate) fn push_definition(
        &mut self,
        subgraph_id: SubgraphId,
        name: &str,
        kind: DefinitionKind,
    ) -> DefinitionId {
        let name = self.strings.intern(name);
        let definition = Definition {
            subgraph_id,
            name,
            kind,
            is_shareable: false,
        };
        let id = DefinitionId(self.definitions.0.push_return_idx(definition));
        self.definition_names.insert((name, subgraph_id), id);
        id
    }
}

pub(crate) type DefinitionWalker<'a> = Walker<'a, DefinitionId>;

impl<'a> DefinitionWalker<'a> {
    fn definition(self) -> &'a Definition {
        &self.subgraphs.definitions.0[self.id.0]
    }

    pub fn name(self) -> StringWalker<'a> {
        self.walk(self.definition().name)
    }

    pub fn kind(self) -> DefinitionKind {
        self.definition().kind
    }

    pub fn is_entity(self) -> bool {
        self.entity_keys().next().is_some()
    }

    pub fn is_shareable(self) -> bool {
        self.definition().is_shareable
    }

    pub fn subgraph(self) -> SubgraphWalker<'a> {
        self.walk(self.definition().subgraph_id)
    }
}

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DefinitionId(usize);

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
    // InputObject,
    // Union,
    // CustomScalar,
}

impl Subgraphs {
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
        let id = push_and_return_id(&mut self.definitions.0, definition, DefinitionId);
        self.definition_names.insert((name, id));
        id
    }
}

pub(crate) type DefinitionWalker<'a> = Walker<'a, DefinitionId>;

impl<'a> DefinitionWalker<'a> {
    fn definition(self) -> &'a Definition {
        &self.subgraphs.definitions.0[self.id.0]
    }

    pub fn name_str(self) -> &'a str {
        self.subgraphs.strings.resolve(self.name())
    }

    pub fn name(self) -> StringId {
        self.definition().name
    }

    pub fn kind(self) -> DefinitionKind {
        self.definition().kind
    }

    pub fn is_entity(self) -> bool {
        self.subgraphs.iter_object_keys(self.id).next().is_some()
    }

    pub fn is_shareable(self) -> bool {
        self.definition().is_shareable
    }

    pub fn subgraph(self) -> SubgraphWalker<'a> {
        self.walk(self.definition().subgraph_id)
    }
}

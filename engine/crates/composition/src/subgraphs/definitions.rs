use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DefinitionId(pub(super) usize);

// Invariant: `definitions` is sorted by `Definition::subgraph_id`. We rely on it for binary search.
#[derive(Default)]
pub(crate) struct Definitions {
    definitions: Vec<Definition>,
    // (Implementee, implementer)
    interface_impls: BTreeSet<(StringId, StringId)>,
}

pub(crate) struct Definition {
    subgraph_id: SubgraphId,
    name: StringId,
    kind: DefinitionKind,
    is_shareable: bool,
    is_external: bool,
    is_interface_object: bool,
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
    pub(crate) fn definition_by_name_id(&self, name: StringId, subgraph_id: SubgraphId) -> Option<DefinitionId> {
        self.definition_names.get(&(name, subgraph_id)).copied()
    }

    pub(crate) fn definition_by_name(&mut self, name: &str, subgraph_id: SubgraphId) -> DefinitionId {
        let interned_name = self.strings.intern(name);
        self.definition_by_name_id(interned_name, subgraph_id).unwrap()
    }

    pub(crate) fn iter_interface_impls(&self) -> impl Iterator<Item = (StringId, StringId)> + '_ {
        self.definitions.interface_impls.iter().copied()
    }

    pub(crate) fn set_external(&mut self, definition_id: DefinitionId) {
        self.definitions.definitions[definition_id.0].is_external = true;
    }

    pub(crate) fn set_interface_object(&mut self, definition_id: DefinitionId) {
        self.definitions.definitions[definition_id.0].is_interface_object = true;
    }

    pub(crate) fn set_shareable(&mut self, definition_id: DefinitionId) {
        self.definitions.definitions[definition_id.0].is_shareable = true;
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
            is_external: false,
            is_interface_object: false,
        };
        let id = DefinitionId(self.definitions.definitions.push_return_idx(definition));
        self.definition_names.insert((name, subgraph_id), id);
        id
    }

    pub(crate) fn push_interface_impl(&mut self, implementer: StringId, implemented_interface: StringId) {
        self.definitions
            .interface_impls
            .insert((implemented_interface, implementer));
    }
}

pub(crate) type DefinitionWalker<'a> = Walker<'a, DefinitionId>;

impl<'a> DefinitionWalker<'a> {
    fn definition(self) -> &'a Definition {
        &self.subgraphs.definitions.definitions[self.id.0]
    }

    pub fn name(self) -> StringWalker<'a> {
        self.walk(self.definition().name)
    }

    pub fn kind(self) -> DefinitionKind {
        self.definition().kind
    }

    pub fn is_interface_object(self) -> bool {
        self.definition().is_interface_object
    }

    pub fn is_shareable(self) -> bool {
        self.definition().is_shareable
    }

    pub fn is_external(self) -> bool {
        self.definition().is_external
    }

    pub fn subgraph(self) -> SubgraphWalker<'a> {
        self.walk(self.definition().subgraph_id)
    }
}

impl<'a> SubgraphWalker<'a> {
    pub(crate) fn definitions(self) -> impl Iterator<Item = DefinitionWalker<'a>> {
        let subgraph_id = self.id;
        let definitions = &self.subgraphs.definitions.definitions;
        let start = definitions.partition_point(|def| def.subgraph_id < self.id);
        let subgraph_definitions = definitions[start..]
            .iter()
            .take_while(move |def| def.subgraph_id == subgraph_id);
        subgraph_definitions
            .enumerate()
            .map(move |(idx, _)| self.walk(DefinitionId(idx + start)))
    }

    pub(crate) fn interface_implementers(self, interface_name: StringId) -> impl Iterator<Item = DefinitionWalker<'a>> {
        self.subgraphs
            .definitions
            .interface_impls
            .iter()
            .filter(move |(implementee, _implementer)| *implementee == interface_name)
            .filter_map(move |(_, implementer)| self.subgraphs.definition_names.get(&(*implementer, self.id)))
            .map(move |id| self.walk(*id))
    }
}

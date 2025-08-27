use super::*;
use std::collections::btree_map;

// Invariant: `definitions` is sorted by `Definition::subgraph_id`. We rely on it for binary search.
#[derive(Default, Debug)]
pub(crate) struct Definitions {
    pub(super) definitions: Vec<Definition>,
    // (Implementee, implementer)
    interface_impls: BTreeSet<(StringId, StringId)>,
    // (Implementee, implementer) -> [subgraph]
    interface_definitions_to_subgraphs: BTreeMap<(StringId, StringId), Vec<SubgraphId>>,
}

#[derive(Debug)]
pub(crate) struct Definition {
    pub(crate) subgraph_id: SubgraphId,
    pub(crate) name: StringId,
    pub(crate) kind: DefinitionKind,
    /// ```graphql,ignore
    /// """
    /// The root query type.
    /// """
    /// ^^^^^^^^^^^^^^^^^^^^
    /// type Query {
    ///   # ...
    /// }
    /// ```
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
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

    pub(crate) fn definition_by_name(&mut self, name: &str, subgraph_id: SubgraphId) -> Option<DefinitionId> {
        let interned_name = self.strings.intern(name);
        self.definition_by_name_id(interned_name, subgraph_id)
    }

    pub(crate) fn iter_definitions_with_name(
        &self,
        name: StringId,
    ) -> impl Iterator<Item = (SubgraphId, DefinitionId)> + '_ {
        self.definition_names
            .range((name, SubgraphId::from(usize::MIN))..(name, SubgraphId::from(usize::MAX)))
            .map(|((_, subgraph_id), definition_id)| (*subgraph_id, *definition_id))
    }

    pub(crate) fn iter_interface_impls(&self) -> impl Iterator<Item = (StringId, StringId)> + '_ {
        self.definitions.interface_impls.iter().copied()
    }

    pub(crate) fn subgraphs_implementing_interface(
        &self,
        implementer: StringId,
        implemented_interface: StringId,
    ) -> impl Iterator<Item = SubgraphId> + '_ {
        self.definitions
            .interface_definitions_to_subgraphs
            .get(&(implementer, implemented_interface))
            .into_iter()
            .flat_map(|subgraphs| subgraphs.iter().copied())
    }

    pub(crate) fn iter_implementers_for_interface(
        &self,
        interface_name: StringId,
    ) -> impl Iterator<Item = StringId> + '_ {
        self.definitions
            .interface_impls
            .range((interface_name, StringId::MIN)..(interface_name, StringId::MAX))
            .map(|(_, implementer)| *implementer)
    }

    pub(crate) fn get_or_push_definition(
        &mut self,
        subgraph_id: SubgraphId,
        name: &str,
        kind: DefinitionKind,
        description: Option<StringId>,
    ) -> DefinitionId {
        let name = self.strings.intern(name);

        if let Some(existing_definition) = self.definition_names.get(&(name, subgraph_id)) {
            return *existing_definition;
        }

        let directive_site_id = self.new_directive_site();

        let definition = Definition {
            subgraph_id,
            name,
            kind,
            description,
            directives: directive_site_id,
        };

        let id = DefinitionId::from(self.definitions.definitions.push_return_idx(definition));
        self.definition_names.insert((name, subgraph_id), id);

        id
    }

    pub(crate) fn push_interface_impl(&mut self, implementer: DefinitionId, implemented_interface: DefinitionId) {
        let implementer_name = self.walk(implementer).name().id;
        let implementee_name = self.walk(implemented_interface).name().id;

        self.definitions
            .interface_impls
            .insert((implementee_name, implementer_name));

        let subgraph_id = self.walk(implementer).subgraph_id();

        match self
            .definitions
            .interface_definitions_to_subgraphs
            .entry((implementee_name, implementer_name))
        {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(vec![subgraph_id]);
            }
            btree_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(subgraph_id);
            }
        }
    }
}

pub(crate) type DefinitionWalker<'a> = Walker<'a, DefinitionId>;

impl<'a> DefinitionWalker<'a> {
    pub fn name(self) -> StringWalker<'a> {
        self.walk(self.view().name)
    }

    pub fn kind(self) -> DefinitionKind {
        self.view().kind
    }

    pub(crate) fn subgraph_id(self) -> SubgraphId {
        self.view().subgraph_id
    }

    pub(crate) fn subgraph(self) -> SubgraphWalker<'a> {
        self.subgraphs.walk_subgraph(self.subgraph_id())
    }
}

impl SubgraphId {
    pub(crate) fn definitions(self, subgraphs: &Subgraphs) -> impl Iterator<Item = View<'_, DefinitionId, Definition>> {
        let definitions = &subgraphs.definitions.definitions;
        let start = definitions.partition_point(|def| def.subgraph_id < self);
        let subgraph_definitions = definitions[start..]
            .iter()
            .take_while(move |def| def.subgraph_id == self);
        subgraph_definitions.enumerate().map(move |(idx, record)| View {
            id: (start + idx).into(),
            record,
        })
    }

    /// Implementers of the given interface in this subgraph.
    pub(crate) fn interface_implementers(
        self,
        subgraphs: &Subgraphs,
        interface_name: StringId,
    ) -> impl Iterator<Item = DefinitionId> {
        subgraphs
            .definitions
            .interface_impls
            .iter()
            .filter(move |(implementee, _implementer)| *implementee == interface_name)
            .filter_map(move |(_, implementer)| subgraphs.definition_names.get(&(*implementer, self)).copied())
    }
}

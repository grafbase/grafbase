use super::*;

impl Subgraphs {
    pub(super) fn is_root_type(&self, SubgraphId(subgraph_idx): SubgraphId, definition: DefinitionId) -> bool {
        let subgraph = &self.subgraphs[subgraph_idx];
        subgraph.query_type == Some(definition)
            || subgraph.mutation_type == Some(definition)
            || subgraph.subscription_type == Some(definition)
    }

    pub(crate) fn iter_subgraphs(&self) -> impl ExactSizeIterator<Item = SubgraphWalker<'_>> {
        self.subgraphs.iter().enumerate().map(|(idx, subgraph)| SubgraphWalker {
            id: (SubgraphId(idx), subgraph),
            subgraphs: self,
        })
    }

    pub(crate) fn push_subgraph(&mut self, name: &str, url: &str) -> SubgraphId {
        let subgraph = Subgraph {
            name: self.strings.intern(name),
            url: self.strings.intern(url),

            query_type: None,
            mutation_type: None,
            subscription_type: None,
        };

        SubgraphId(self.subgraphs.push_return_idx(subgraph))
    }

    pub(crate) fn set_query_type(&mut self, subgraph: SubgraphId, query_type: DefinitionId) {
        self.subgraphs[subgraph.0].query_type = Some(query_type);
    }

    pub(crate) fn set_mutation_type(&mut self, subgraph: SubgraphId, mutation_type: DefinitionId) {
        self.subgraphs[subgraph.0].mutation_type = Some(mutation_type);
    }

    pub(crate) fn set_subscription_type(&mut self, subgraph: SubgraphId, subscription_type: DefinitionId) {
        self.subgraphs[subgraph.0].subscription_type = Some(subscription_type);
    }

    pub(crate) fn walk_subgraph(&self, subgraph_id: SubgraphId) -> SubgraphWalker<'_> {
        SubgraphWalker {
            id: (subgraph_id, &self.subgraphs[subgraph_id.0]),
            subgraphs: self,
        }
    }
}

pub(crate) struct Subgraph {
    /// The name of the subgraph. It is not contained in the GraphQL schema of the subgraph, it
    /// only makes sense within a project.
    name: StringId,
    url: StringId,

    query_type: Option<DefinitionId>,
    mutation_type: Option<DefinitionId>,
    subscription_type: Option<DefinitionId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SubgraphId(usize);

impl SubgraphId {
    pub(crate) const MIN: SubgraphId = SubgraphId(usize::MIN);
    pub(crate) const MAX: SubgraphId = SubgraphId(usize::MAX);

    pub(crate) fn idx(self) -> usize {
        self.0
    }
}

pub(crate) type SubgraphWalker<'a> = Walker<'a, (SubgraphId, &'a Subgraph)>;

impl<'a> SubgraphWalker<'a> {
    pub(crate) fn subgraph_id(self) -> SubgraphId {
        let (id, _) = self.id;
        id
    }

    fn subgraph(self) -> &'a Subgraph {
        let (_, subgraph) = self.id;
        subgraph
    }

    pub(crate) fn query_type(self) -> Option<DefinitionWalker<'a>> {
        self.subgraph().query_type.map(|id| self.walk(id))
    }

    pub(crate) fn mutation_type(self) -> Option<DefinitionWalker<'a>> {
        self.subgraph().mutation_type.map(|id| self.walk(id))
    }

    pub(crate) fn subscription_type(self) -> Option<DefinitionWalker<'a>> {
        self.subgraph().subscription_type.map(|id| self.walk(id))
    }

    pub(crate) fn name(self) -> StringWalker<'a> {
        self.walk(self.subgraph().name)
    }

    pub(crate) fn url(self) -> StringWalker<'a> {
        self.walk(self.subgraph().url)
    }
}

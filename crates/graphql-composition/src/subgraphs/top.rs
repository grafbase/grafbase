use super::*;

impl Subgraphs {
    pub(super) fn is_root_type(&self, subgraph_id: SubgraphId, definition: DefinitionId) -> bool {
        let subgraph = &self.subgraphs[usize::from(subgraph_id)];
        subgraph.query_type == Some(definition)
            || subgraph.mutation_type == Some(definition)
            || subgraph.subscription_type == Some(definition)
    }

    pub(crate) fn iter_subgraphs(&self) -> impl ExactSizeIterator<Item = SubgraphWalker<'_>> {
        self.subgraphs.iter().enumerate().map(|(idx, subgraph)| SubgraphWalker {
            id: (SubgraphId::from(idx), subgraph),
            subgraphs: self,
        })
    }

    pub(crate) fn iter_subgraph_views(&self) -> impl ExactSizeIterator<Item = View<'_, SubgraphId, Subgraph>> {
        self.subgraphs
            .iter()
            .enumerate()
            .map(|(idx, record)| View { id: idx.into(), record })
    }

    pub(crate) fn push_subgraph(&mut self, name: &str, url: Option<&str>) -> SubgraphId {
        let url = url.map(|url| self.strings.intern(url));

        let subgraph = Subgraph {
            name: self.strings.intern(name),
            url,

            query_type: None,
            mutation_type: None,
            subscription_type: None,
        };

        SubgraphId::from(self.subgraphs.push_return_idx(subgraph))
    }

    pub(crate) fn set_query_type(&mut self, subgraph: SubgraphId, query_type: DefinitionId) {
        self.subgraphs[usize::from(subgraph)].query_type = Some(query_type);
    }

    pub(crate) fn set_mutation_type(&mut self, subgraph: SubgraphId, mutation_type: DefinitionId) {
        self.subgraphs[usize::from(subgraph)].mutation_type = Some(mutation_type);
    }

    pub(crate) fn set_subscription_type(&mut self, subgraph: SubgraphId, subscription_type: DefinitionId) {
        self.subgraphs[usize::from(subgraph)].subscription_type = Some(subscription_type);
    }

    pub(crate) fn walk_subgraph(&self, subgraph_id: SubgraphId) -> SubgraphWalker<'_> {
        SubgraphWalker {
            id: (subgraph_id, &self.subgraphs[usize::from(subgraph_id)]),
            subgraphs: self,
        }
    }
}

pub(crate) struct Subgraph {
    /// The name of the subgraph. It is not contained in the GraphQL schema of the subgraph, it
    /// only makes sense within a project.
    pub(crate) name: StringId,
    pub(crate) url: Option<StringId>,

    pub(crate) query_type: Option<DefinitionId>,
    pub(crate) mutation_type: Option<DefinitionId>,
    pub(crate) subscription_type: Option<DefinitionId>,
}

impl Subgraph {
    pub(crate) fn is_virtual(&self) -> bool {
        self.url.is_none()
    }
}

impl SubgraphId {
    pub(crate) fn idx(self) -> usize {
        self.into()
    }
}

pub(crate) type SubgraphWalker<'a> = Walker<'a, (SubgraphId, &'a Subgraph)>;

impl<'a> SubgraphWalker<'a> {
    #[expect(unused)]
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

    pub(crate) fn url(self) -> Option<StringWalker<'a>> {
        self.subgraph().url.map(|url| self.walk(url))
    }
}

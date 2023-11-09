//! The public API for traversing [Subgraphs].

use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Walker<'a, Id> {
    pub(crate) id: Id,
    pub(crate) subgraphs: &'a Subgraphs,
}

impl<'a, Id> Walker<'a, Id> {
    pub(crate) fn walk<T>(self, other: T) -> Walker<'a, T> {
        self.subgraphs.walk(other)
    }
}

pub(crate) type SubgraphWalker<'a> = Walker<'a, SubgraphId>;

impl<'a> SubgraphWalker<'a> {
    fn subgraph(self) -> &'a Subgraph {
        &self.subgraphs.subgraphs[self.id.0]
    }

    pub(crate) fn name(self) -> StringWalker<'a> {
        self.walk(self.subgraph().name)
    }

    pub(crate) fn url(self) -> StringWalker<'a> {
        self.walk(self.subgraph().name) // TODO: take the url as input
    }
}

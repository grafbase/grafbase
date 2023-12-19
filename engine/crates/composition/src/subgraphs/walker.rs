//! The public API for traversing [Subgraphs].

use super::Subgraphs;

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

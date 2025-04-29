//! The public API for traversing [Subgraphs].

use super::{Subgraphs, View};
use std::ops::Index;

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

impl<'a, Id, Record> Walker<'a, Id>
where
    Id: Copy,
    Subgraphs: Index<Id, Output = Record>,
{
    pub(crate) fn view(&self) -> View<'a, Id, Record> {
        self.subgraphs.at(self.id)
    }
}

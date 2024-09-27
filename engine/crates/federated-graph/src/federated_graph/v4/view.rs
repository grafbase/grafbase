//! Convenient methods and helper types to navigate a [FederatedGraph].

use std::ops::Index;

use super::{FederatedGraph, StringId};

pub struct View<Id, Record> {
    id: Id,
    record: Record,
}

impl<Id, Record> View<Id, Record>
where
    Id: Copy,
{
    pub fn id(&self) -> Id {
        self.id
    }
}

impl<Id, Record> AsRef<Record> for View<Id, Record> {
    fn as_ref(&self) -> &Record {
        &self.record
    }
}

impl<Id, Record> std::ops::Deref for View<Id, Record> {
    type Target = Record;

    fn deref(&self) -> &Self::Target {
        &self.record
    }
}

pub struct ViewNested<'a, Record> {
    graph: &'a FederatedGraph,
    record: &'a Record,
}

impl<'a, Record> ViewNested<'a, Record> {
    /// Continue navigating with the next ID.
    pub fn through<Id, Next>(&self, next: impl FnOnce(&Record) -> Id) -> ViewNested<'a, Next>
    where
        FederatedGraph: Index<Id, Output = Next>,
    {
        ViewNested {
            graph: self.graph,
            record: &self.graph[next(self.record)],
        }
    }

    /// Resolve a [StringId].
    pub fn str(&self, next: impl FnOnce(&Record) -> StringId) -> &'a str {
        self.graph[next(self.record)].as_str()
    }

    /// View the record with the provided ID.
    pub fn view<Id, Next>(&self, next: impl FnOnce(&Record) -> Id) -> View<Id, &'a Next>
    where
        Id: Copy,
        FederatedGraph: Index<Id, Output = Next>,
    {
        self.graph.view(next(self.record))
    }
}

impl FederatedGraph {
    /// Start navigating the graph from the given ID. Returns a [ViewNested] that exposes further steps.
    pub fn through<Id, Record>(&self, id: Id) -> ViewNested<'_, Record>
    where
        FederatedGraph: Index<Id, Output = Record>,
    {
        ViewNested {
            graph: self,
            record: &self[id],
        }
    }

    /// Resolve a [StringId].
    pub fn str(&self, id: StringId) -> &str {
        self[id].as_str()
    }

    /// View the record with the given ID.
    pub fn view<Id, Record>(&self, id: Id) -> View<Id, &Record>
    where
        Id: Copy,
        FederatedGraph: Index<Id, Output = Record>,
    {
        View { id, record: &self[id] }
    }
}

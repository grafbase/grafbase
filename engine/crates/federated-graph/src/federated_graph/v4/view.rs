//! Convenient methods and helper types to navigate a [FederatedGraph].

use std::ops::{Deref, Index};

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

pub struct ViewNested<'a, Id, Record> {
    graph: &'a FederatedGraph,
    view: View<Id, &'a Record>,
}

impl<'a, Id, Record> Deref for ViewNested<'a, Id, Record> {
    type Target = View<Id, &'a Record>;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'a, Id, Record> AsRef<View<Id, &'a Record>> for ViewNested<'a, Id, Record> {
    fn as_ref(&self) -> &View<Id, &'a Record> {
        &self.view
    }
}

impl<'a, Id, Record> AsRef<Record> for ViewNested<'a, Id, Record> {
    fn as_ref(&self) -> &Record {
        self.view.as_ref()
    }
}

impl<'a, Id, Record> ViewNested<'a, Id, Record> {
    /// Continue navigating with the next ID.
    pub fn then<NextId, NextRecord>(&self, next: impl FnOnce(&Record) -> NextId) -> ViewNested<'a, NextId, NextRecord>
    where
        NextId: Copy,
        FederatedGraph: Index<NextId, Output = NextRecord>,
    {
        self.graph.at(next(self.view.record))
    }
}

impl FederatedGraph {
    /// Start navigating the graph from the given ID. Returns a [ViewNested] that exposes further steps.
    pub fn at<Id, Record>(&self, id: Id) -> ViewNested<'_, Id, Record>
    where
        Id: Copy,
        FederatedGraph: Index<Id, Output = Record>,
    {
        ViewNested {
            graph: self,
            view: self.view(id),
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

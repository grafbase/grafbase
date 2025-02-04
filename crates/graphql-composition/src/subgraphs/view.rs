use super::Subgraphs;
use std::ops::{Deref, Index};

pub(crate) struct View<'a, Id, Record> {
    pub(crate) id: Id,
    pub(crate) record: &'a Record,
}

impl<Id, Record> Deref for View<'_, Id, Record> {
    type Target = Record;

    fn deref(&self) -> &Self::Target {
        self.record
    }
}

impl Subgraphs {
    pub(crate) fn at<Id, Record>(&self, id: Id) -> View<'_, Id, Record>
    where
        Id: Copy,
        Subgraphs: Index<Id, Output = Record>,
    {
        View { id, record: &self[id] }
    }
}

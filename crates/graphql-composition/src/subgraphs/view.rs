use super::Subgraphs;
use std::ops::{Deref, Index};

#[derive(Debug)]
pub(crate) struct View<'a, Id, Record> {
    pub(crate) id: Id,
    pub(crate) record: &'a Record,
}

impl<Id, Record> Clone for View<'_, Id, Record>
where
    Id: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            record: self.record,
        }
    }
}

impl<Id, Record> Copy for View<'_, Id, Record> where Id: Copy {}

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

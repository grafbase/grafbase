use std::ops::{Deref, Index};

use super::*;

pub(crate) struct View<'a, Id, Record> {
    pub(crate) id: Id,
    pub(crate) record: &'a Record,
}

impl<'a, Id, Record> View<'a, Id, Record> {
    pub(crate) fn new(id: impl Into<Id>, record: &'a Record) -> Self {
        Self { id: id.into(), record }
    }
}

impl<Id, Record> Deref for View<'_, Id, Record> {
    type Target = Record;

    fn deref(&self) -> &Self::Target {
        self.record
    }
}

impl TranslatedSchema {
    pub(crate) fn view<'a, Id, Record>(&'a self, id: Id) -> View<'a, Id, Record>
    where
        Id: Copy,
        Self: Index<Id, Output = Record>,
    {
        View { id, record: &self[id] }
    }
}

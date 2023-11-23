use schema::SchemaWalker;

use super::BoundSelectionSetWalker;
use crate::request::{BoundFragmentSpread, Operation};

pub struct BoundFragmentSpreadWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub(in crate::request) inner: &'a BoundFragmentSpread,
}

impl<'a> BoundFragmentSpreadWalker<'a> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        BoundSelectionSetWalker {
            schema: self.schema,
            operation: self.operation,
            id: self.inner.selection_set_id,
        }
    }
}

impl<'a> std::fmt::Debug for BoundFragmentSpreadWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.inner.fragment_id];
        f.debug_struct("BoundFragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

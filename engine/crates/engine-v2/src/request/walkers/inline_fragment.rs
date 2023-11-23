use schema::SchemaWalker;

use super::BoundSelectionSetWalker;
use crate::request::{BoundInlineFragment, Operation};

pub struct BoundInlineFragmentWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub(in crate::request) inner: &'a BoundInlineFragment,
}

impl<'a> BoundInlineFragmentWalker<'a> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        BoundSelectionSetWalker {
            schema: self.schema,
            operation: self.operation,
            id: self.inner.selection_set_id,
        }
    }
}

impl<'a> std::fmt::Debug for BoundInlineFragmentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

use schema::SchemaWalker;

use super::{BoundSelectionSetWalker, FlattenFieldsIterator};
use crate::request::{BoundFragmentSpread, Operation, ResolvedTypeCondition};

pub struct BoundFragmentSpreadWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub(in crate::request) inner: &'a BoundFragmentSpread,
}

impl<'a> BoundFragmentSpreadWalker<'a> {
    pub(super) fn nested_fields(
        &self,
        parent_type_condition: Option<ResolvedTypeCondition>,
    ) -> FlattenFieldsIterator<'a> {
        let fragment_definition = &self.operation[self.inner.fragment_id];
        FlattenFieldsIterator {
            resolved_type_condition: ResolvedTypeCondition::merge(
                parent_type_condition,
                Some(fragment_definition.type_condition.resolve(&self.schema)),
            ),
            selections: self.selection_set().into_iter(),
            nested: None,
        }
    }

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

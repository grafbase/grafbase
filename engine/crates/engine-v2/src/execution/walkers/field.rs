use engine_parser::Pos;
use schema::FieldId;

use super::{FieldArgumentWalker, SelectionSetWalker};
use crate::request::{BoundField, BoundFieldDefinition};

pub struct FieldWalker<'a> {
    pub(super) ctx: super::WalkerContext<'a, FieldId>,
    pub(super) bound_field: BoundField,
    pub(super) definition: &'a BoundFieldDefinition,
}

impl<'a> FieldWalker<'a> {
    pub fn location(&self) -> Pos {
        self.definition.name_location
    }

    pub fn response_key(&self) -> &str {
        &self.ctx.operation.response_keys[self.definition.response_key]
    }

    pub fn bound_arguments<'s>(&'s self) -> impl ExactSizeIterator<Item = FieldArgumentWalker<'s>> + 's
    where
        'a: 's,
    {
        let ctx = self.ctx;
        self.definition
            .arguments
            .iter()
            .map(move |argument| FieldArgumentWalker {
                input_value: ctx.schema_walker.walk(argument.input_value_id),
                variables: ctx.variables,
                argument,
            })
    }

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        self.bound_field.selection_set_id.and_then(|id| {
            if self.ctx.attribution.selection_set(id) {
                Some(SelectionSetWalker {
                    ctx: self.ctx.walk(()),
                    merged_selection_set_ids: vec![id],
                })
            } else {
                None
            }
        })
    }
}

impl<'a> std::ops::Deref for FieldWalker<'a> {
    type Target = schema::FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx.schema_walker
    }
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldWalker")
            .field("name", &self.name())
            .field("arguments", &self.bound_arguments().collect::<Vec<_>>())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

use engine_parser::Pos;
use schema::FieldId;

use super::{FieldArgumentWalker, SelectionSetWalker};
use crate::request::BoundField;

pub struct FieldWalker<'a> {
    pub(super) ctx: super::WalkerContext<'a, FieldId>,
    pub(super) bound_field: BoundField,
}

impl<'a> FieldWalker<'a> {
    pub fn location(&self) -> Pos {
        self.ctx.plan.operation[self.bound_field.definition_id].name_location
    }

    pub fn bound_arguments<'s>(&'s self) -> impl ExactSizeIterator<Item = FieldArgumentWalker<'s>> + 's
    where
        'a: 's,
    {
        let ctx = self.ctx;
        self.ctx.plan.operation[self.bound_field.definition_id]
            .arguments
            .iter()
            .map(move |argument| FieldArgumentWalker {
                ctx: ctx.walk(argument.input_value_id),
                argument,
            })
    }

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        if self.ctx.plan.attribution[self.bound_field.selection_set_id].contains(&self.ctx.plan_id) {
            Some(SelectionSetWalker {
                ctx: self.ctx.walk(()),
                id: self.bound_field.selection_set_id,
            })
        } else {
            None
        }
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

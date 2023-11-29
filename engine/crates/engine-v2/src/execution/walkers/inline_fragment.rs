use engine_parser::Pos;

use super::{SelectionSetWalker, WalkerContext};
use crate::request::{BoundInlineFragment, TypeCondition};

pub struct InlineFragmentWalker<'a> {
    pub(super) ctx: WalkerContext<'a, ()>,
    pub(super) inner: &'a BoundInlineFragment,
}

impl<'a> InlineFragmentWalker<'a> {
    pub fn location(&self) -> Pos {
        self.inner.location
    }

    pub fn type_condition_name(&self) -> Option<&str> {
        self.inner.type_condition.map(|cond| match cond {
            TypeCondition::Interface(interface_id) => self.ctx.schema_walker.walk(interface_id).name(),
            TypeCondition::Object(object_id) => self.ctx.schema_walker.walk(object_id).name(),
            TypeCondition::Union(union_id) => self.ctx.schema_walker.walk(union_id).name(),
        })
    }

    pub fn selection_set(&self) -> SelectionSetWalker<'a> {
        SelectionSetWalker {
            ctx: self.ctx,
            id: self.inner.selection_set_id,
        }
    }
}

impl<'a> std::fmt::Debug for InlineFragmentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InlineFragmentWalker")
            .field(&self.selection_set())
            .finish()
    }
}

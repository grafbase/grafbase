use std::ops::Deref;

use engine_parser::Pos;

use super::{SelectionSetWalker, WalkerContext};
use crate::request::{BoundFragmentDefinition, BoundFragmentSpread, TypeCondition};

pub struct FragmentSpreadWalker<'a> {
    pub(super) ctx: WalkerContext<'a, ()>,
    pub(super) inner: &'a BoundFragmentSpread,
}

impl<'a> FragmentSpreadWalker<'a> {
    pub fn location(&self) -> Pos {
        self.inner.location
    }

    pub fn selection_set(&self) -> SelectionSetWalker<'a> {
        SelectionSetWalker {
            ctx: self.ctx,
            id: self.inner.selection_set_id,
        }
    }

    pub fn fragment(&self) -> FragmentDefinitionWalker<'a> {
        FragmentDefinitionWalker {
            ctx: self.ctx,
            definition: &self.ctx.plan.operation[self.inner.fragment_id],
        }
    }
}

pub struct FragmentDefinitionWalker<'a> {
    pub(super) ctx: WalkerContext<'a, ()>,
    pub(super) definition: &'a BoundFragmentDefinition,
}

impl<'a> FragmentDefinitionWalker<'a> {
    pub fn type_condition_name(&self) -> &str {
        match self.type_condition {
            TypeCondition::Interface(interface_id) => self.ctx.schema_walker.walk(interface_id).name(),
            TypeCondition::Object(object_id) => self.ctx.schema_walker.walk(object_id).name(),
            TypeCondition::Union(union_id) => self.ctx.schema_walker.walk(union_id).name(),
        }
    }
}

impl<'a> Deref for FragmentDefinitionWalker<'a> {
    type Target = BoundFragmentDefinition;

    fn deref(&self) -> &Self::Target {
        self.definition
    }
}

impl<'a> std::fmt::Debug for FragmentSpreadWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.ctx.plan.operation[self.inner.fragment_id];
        f.debug_struct("FragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

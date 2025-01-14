//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{
    generated::{ExecutableDirective, ExecutableDirectiveId},
    prelude::*,
    SelectionSet, SelectionSetRecord,
};
use schema::{CompositeType, CompositeTypeId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type InlineFragment @meta(module: "selection/inline_fragment") @indexed(id_size: "u16") {
///   type_condition: CompositeType
///   directives: [ExecutableDirective!]!
///   selection_set: SelectionSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InlineFragmentRecord {
    pub type_condition_id: Option<CompositeTypeId>,
    pub directive_ids: Vec<ExecutableDirectiveId>,
    pub selection_set_record: SelectionSetRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct InlineFragmentId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct InlineFragment<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: InlineFragmentId,
}

impl std::ops::Deref for InlineFragment<'_> {
    type Target = InlineFragmentRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> InlineFragment<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a InlineFragmentRecord {
        &self.ctx.operation[self.id]
    }
    pub fn type_condition(&self) -> Option<CompositeType<'a>> {
        self.as_ref().type_condition_id.walk(self.ctx)
    }
    pub fn directives(&self) -> impl Iter<Item = ExecutableDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.ctx)
    }
    pub fn selection_set(&self) -> SelectionSet<'a> {
        self.as_ref().selection_set_record.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for InlineFragmentId {
    type Walker<'w>
        = InlineFragment<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        InlineFragment {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for InlineFragment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineFragment")
            .field("type_condition", &self.type_condition())
            .field("directives", &self.directives())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

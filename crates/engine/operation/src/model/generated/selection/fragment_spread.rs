//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{
    generated::{ExecutableDirective, ExecutableDirectiveId, Fragment, FragmentId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FragmentSpread @meta(module: "selection/fragment_spread") @indexed(id_size: "u16") {
///   directives: [ExecutableDirective!]!
///   fragment: Fragment!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FragmentSpreadRecord {
    pub directive_ids: Vec<ExecutableDirectiveId>,
    pub fragment_id: FragmentId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FragmentSpreadId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct FragmentSpread<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: FragmentSpreadId,
}

impl std::ops::Deref for FragmentSpread<'_> {
    type Target = FragmentSpreadRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> FragmentSpread<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FragmentSpreadRecord {
        &self.ctx.operation[self.id]
    }
    pub fn directives(&self) -> impl Iter<Item = ExecutableDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.ctx)
    }
    pub fn fragment(&self) -> Fragment<'a> {
        self.fragment_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for FragmentSpreadId {
    type Walker<'w>
        = FragmentSpread<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FragmentSpread {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for FragmentSpread<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FragmentSpread")
            .field("directives", &self.directives())
            .field("fragment", &self.fragment())
            .finish()
    }
}

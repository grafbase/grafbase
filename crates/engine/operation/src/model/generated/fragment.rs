//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{prelude::*, SelectionSet, SelectionSetRecord};
use schema::{CompositeType, CompositeTypeId};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type Fragment @meta(module: "fragment") @indexed(id_size: "u16") {
///   type_condition: CompositeType!
///   selection_set: SelectionSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FragmentRecord {
    pub type_condition_id: CompositeTypeId,
    pub selection_set_record: SelectionSetRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FragmentId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct Fragment<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: FragmentId,
}

impl std::ops::Deref for Fragment<'_> {
    type Target = FragmentRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> Fragment<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FragmentRecord {
        &self.ctx.operation[self.id]
    }
    pub fn type_condition(&self) -> CompositeType<'a> {
        self.type_condition_id.walk(self.ctx)
    }
    pub fn selection_set(&self) -> SelectionSet<'a> {
        self.as_ref().selection_set_record.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for FragmentId {
    type Walker<'w>
        = Fragment<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        Fragment {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for Fragment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fragment")
            .field("type_condition", &self.type_condition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

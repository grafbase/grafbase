//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    PartitionDataField, PartitionDataFieldId, PartitionTypenameField, PartitionTypenameFieldId, prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type PartitionSelectionSet @meta(module: "selection_set", derive: ["Default"]) @copy {
///   data_fields_ordered_by_parent_entity_then_key: [PartitionDataField!]!
///     @field(record_field_name: "data_field_ids_ordered_by_parent_entity_then_key")
///   typename_fields: [PartitionTypenameField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
pub(crate) struct PartitionSelectionSetRecord {
    pub data_field_ids_ordered_by_parent_entity_then_key: IdRange<PartitionDataFieldId>,
    pub typename_field_ids: IdRange<PartitionTypenameFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct PartitionSelectionSet<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) item: PartitionSelectionSetRecord,
}

impl std::ops::Deref for PartitionSelectionSet<'_> {
    type Target = PartitionSelectionSetRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[allow(unused)]
impl<'a> PartitionSelectionSet<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &PartitionSelectionSetRecord {
        &self.item
    }
    pub(crate) fn data_fields_ordered_by_parent_entity_then_key(
        &self,
    ) -> impl Iter<Item = PartitionDataField<'a>> + 'a {
        self.as_ref()
            .data_field_ids_ordered_by_parent_entity_then_key
            .walk(self.ctx)
    }
    pub(crate) fn typename_fields(&self) -> impl Iter<Item = PartitionTypenameField<'a>> + 'a {
        self.as_ref().typename_field_ids.walk(self.ctx)
    }
}

#[allow(unused)]
impl<'a> Walk<CachedOperationContext<'a>> for PartitionSelectionSetRecord {
    type Walker<'w>
        = PartitionSelectionSet<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PartitionSelectionSet {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for PartitionSelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartitionSelectionSet")
            .field(
                "data_fields_ordered_by_parent_entity_then_key",
                &self.data_fields_ordered_by_parent_entity_then_key(),
            )
            .field("typename_fields", &self.typename_fields())
            .finish()
    }
}

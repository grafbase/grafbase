//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    DataField, DataFieldId, LookupField, LookupFieldId, TypenameField, TypenameFieldId, prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type PartitionSelectionSet @meta(module: "selection_set", derive: ["Default"], debug: false) @copy {
///   data_fields_ordered_by_parent_entity_then_key: [DataField!]!
///     @field(record_field_name: "data_field_ids_ordered_by_parent_entity_then_key")
///   typename_fields: [TypenameField!]!
///   lookup_fields: [LookupField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
pub(crate) struct PartitionSelectionSetRecord {
    pub data_field_ids_ordered_by_parent_entity_then_key: IdRange<DataFieldId>,
    pub typename_field_ids: IdRange<TypenameFieldId>,
    pub lookup_field_ids: IdRange<LookupFieldId>,
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
    pub(crate) fn data_fields_ordered_by_parent_entity_then_key(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.as_ref()
            .data_field_ids_ordered_by_parent_entity_then_key
            .walk(self.ctx)
    }
    pub(crate) fn typename_fields(&self) -> impl Iter<Item = TypenameField<'a>> + 'a {
        self.as_ref().typename_field_ids.walk(self.ctx)
    }
    pub(crate) fn lookup_fields(&self) -> impl Iter<Item = LookupField<'a>> + 'a {
        self.as_ref().lookup_field_ids.walk(self.ctx)
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

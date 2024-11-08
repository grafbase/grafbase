//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{
    generated::{DataPlanField, DataPlanFieldId, TypenamePlanField, TypenamePlanFieldId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type PlanSelectionSet @meta(module: "selection_set") @copy {
///   data_fields: [DataPlanField!]!
///   typename_fields: [TypenamePlanField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub(crate) struct PlanSelectionSetRecord {
    pub data_field_ids: IdRange<DataPlanFieldId>,
    pub typename_field_ids: IdRange<TypenamePlanFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct PlanSelectionSet<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) item: PlanSelectionSetRecord,
}

impl std::ops::Deref for PlanSelectionSet<'_> {
    type Target = PlanSelectionSetRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[allow(unused)]
impl<'a> PlanSelectionSet<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &PlanSelectionSetRecord {
        &self.item
    }
    pub(crate) fn data_fields(&self) -> impl Iter<Item = DataPlanField<'a>> + 'a {
        self.data_field_ids.walk(self.ctx)
    }
    pub(crate) fn typename_fields(&self) -> impl Iter<Item = TypenamePlanField<'a>> + 'a {
        self.typename_field_ids.walk(self.ctx)
    }
}

#[allow(unused)]
impl<'a> Walk<PlanContext<'a>> for PlanSelectionSetRecord {
    type Walker<'w> = PlanSelectionSet<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PlanSelectionSet { ctx, item: self }
    }
}

impl std::fmt::Debug for PlanSelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanSelectionSet")
            .field("data_fields", &self.data_fields())
            .field("typename_fields", &self.typename_fields())
            .finish()
    }
}

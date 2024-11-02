//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{
    generated::{DataField, DataFieldId, TypenameField, TypenameFieldId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SelectionSet @meta(module: "selection_set") @copy {
///   data_fields: [DataField!]!
///   typename_fields: [TypenameField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub(crate) struct SelectionSetRecord {
    pub data_field_ids: IdRange<DataFieldId>,
    pub typename_field_ids: IdRange<TypenameFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct SelectionSet<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) item: SelectionSetRecord,
}

impl std::ops::Deref for SelectionSet<'_> {
    type Target = SelectionSetRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[allow(unused)]
impl<'a> SelectionSet<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &SelectionSetRecord {
        &self.item
    }
    pub(crate) fn data_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.data_field_ids.walk(self.ctx)
    }
    pub(crate) fn typename_fields(&self) -> impl Iter<Item = TypenameField<'a>> + 'a {
        self.typename_field_ids.walk(self.ctx)
    }
}

#[allow(unused)]
impl<'a> Walk<PlanContext<'a>> for SelectionSetRecord {
    type Walker<'w> = SelectionSet<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SelectionSet { ctx, item: self }
    }
}

impl std::fmt::Debug for SelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("data_fields", &self.data_fields())
            .field("typename_fields", &self.typename_fields())
            .finish()
    }
}

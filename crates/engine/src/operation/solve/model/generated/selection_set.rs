//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/solved_operation.graphql
use crate::operation::solve::model::{
    generated::{DataField, DataFieldId, TypenameField, TypenameFieldId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SelectionSet @meta(module: "selection_set") @copy {
///   data_fields_ordered_by_parent_entity_id_then_key: [DataField!]!
///     @field(record_field_name: "data_field_ids_ordered_by_parent_entity_id_then_key")
///   typename_fields_ordered_by_type_condition_id_then_key: [TypenameField!]!
///     @field(record_field_name: "typename_field_ids_ordered_by_type_condition_id_then_key")
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub(crate) struct SelectionSetRecord {
    pub data_field_ids_ordered_by_parent_entity_id_then_key: IdRange<DataFieldId>,
    pub typename_field_ids_ordered_by_type_condition_id_then_key: IdRange<TypenameFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct SelectionSet<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
    pub(in crate::operation::solve::model) item: SelectionSetRecord,
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
    pub(crate) fn data_fields_ordered_by_parent_entity_id_then_key(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.data_field_ids_ordered_by_parent_entity_id_then_key.walk(self.ctx)
    }
    pub(crate) fn typename_fields_ordered_by_type_condition_id_then_key(
        &self,
    ) -> impl Iter<Item = TypenameField<'a>> + 'a {
        self.typename_field_ids_ordered_by_type_condition_id_then_key
            .walk(self.ctx)
    }
}

#[allow(unused)]
impl<'a> Walk<SolvedOperationContext<'a>> for SelectionSetRecord {
    type Walker<'w> = SelectionSet<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SelectionSet {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for SelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field(
                "data_fields_ordered_by_parent_entity_id_then_key",
                &self.data_fields_ordered_by_parent_entity_id_then_key(),
            )
            .field(
                "typename_fields_ordered_by_type_condition_id_then_key",
                &self.typename_fields_ordered_by_type_condition_id_then_key(),
            )
            .finish()
    }
}

//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::{
    generated::{Field, FieldArgument, FieldArgumentId, FieldId},
    prelude::*,
    FieldRefId,
};
use schema::{FieldDefinition, FieldDefinitionId};
use walker::{Iter, Walk};

/// In opposition to a __typename field this field does retrieve data from a subgraph
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DataField @meta(module: "field/data") {
///   key: PositionedResponseKey!
///   location: Location!
///   definition: FieldDefinition!
///   arguments: [FieldArgument!]!
///   selection_set_fields: [Field!]!
///   required_fields: [FieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DataFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub selection_set_field_ids: IdRange<FieldId>,
    pub required_field_ids: IdRange<FieldRefId>,
}

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct DataField<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) ref_: &'a DataFieldRecord,
}

impl std::ops::Deref for DataField<'_> {
    type Target = DataFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> DataField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a DataFieldRecord {
        self.ref_
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.argument_ids.walk(self.ctx)
    }
    pub(crate) fn selection_set_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.selection_set_field_ids.walk(self.ctx)
    }
    pub(crate) fn required_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.required_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for &DataFieldRecord {
    type Walker<'w> = DataField<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DataField { ctx, ref_: self }
    }
}

impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataField")
            .field("key", &self.key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set_fields", &self.selection_set_fields())
            .field("required_fields", &self.required_fields())
            .finish()
    }
}

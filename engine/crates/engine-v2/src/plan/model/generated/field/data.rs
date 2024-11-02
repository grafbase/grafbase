//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{
    generated::{
        FieldArgument, FieldArgumentId, ResponseObjectSetDefinition, ResponseObjectSetDefinitionId, SelectionSet,
        SelectionSetRecord,
    },
    prelude::*,
    DataFieldRefId, FieldShapeRefId,
};
use schema::{FieldDefinition, FieldDefinitionId, RequiredField, RequiredFieldId};
use walker::{Iter, Walk};

/// In opposition to a __typename field this field does retrieve data from a subgraph
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DataField @meta(module: "field/data") @indexed(id_size: "u32") {
///   key: PositionedResponseKey!
///   location: Location!
///   definition: FieldDefinition!
///   arguments: [FieldArgument!]!
///   "Fields required either by @requires, @authorized, etc."
///   required_fields: [DataFieldRef!]!
///   "All field shape ids generated for this field"
///   shape_ids: [FieldShapeRefId!]!
///   output: ResponseObjectSetDefinition
///   selection_set: SelectionSet!
///   "Whether __typename should be requested from the subgraph for this selection set"
///   selection_set_requires_typename: Boolean!
///   matching_requirement: RequiredField
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DataFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    /// Fields required either by @requires, @authorized, etc.
    pub required_field_ids: IdRange<DataFieldRefId>,
    /// All field shape ids generated for this field
    pub shape_ids: IdRange<FieldShapeRefId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: SelectionSetRecord,
    /// Whether __typename should be requested from the subgraph for this selection set
    pub selection_set_requires_typename: bool,
    pub matching_requirement_id: Option<RequiredFieldId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DataFieldId(std::num::NonZero<u32>);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct DataField<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) id: DataFieldId,
}

impl std::ops::Deref for DataField<'_> {
    type Target = DataFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> DataField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a DataFieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> DataFieldId {
        self.id
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.argument_ids.walk(self.ctx)
    }
    /// Fields required either by @requires, @authorized, etc.
    pub(crate) fn required_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.required_field_ids.walk(self.ctx)
    }
    pub(crate) fn output(&self) -> Option<ResponseObjectSetDefinition<'a>> {
        self.output_id.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> SelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn matching_requirement(&self) -> Option<RequiredField<'a>> {
        self.matching_requirement_id.walk(self.ctx.schema)
    }
}

impl<'a> Walk<PlanContext<'a>> for DataFieldId {
    type Walker<'w> = DataField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DataField { ctx, id: self }
    }
}

impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataField")
            .field("key", &self.key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("required_fields", &self.required_fields())
            .field("shape_ids", &self.shape_ids)
            .field("output", &self.output())
            .field("selection_set", &self.selection_set())
            .field("selection_set_requires_typename", &self.selection_set_requires_typename)
            .field("matching_requirement", &self.matching_requirement())
            .finish()
    }
}

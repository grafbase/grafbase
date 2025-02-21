//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{
    Location, ResponseKey, SelectionSet, SelectionSetRecord,
    generated::{ExecutableDirective, ExecutableDirectiveId, FieldArgument, FieldArgumentId},
    prelude::*,
};
use schema::{FieldDefinition, FieldDefinitionId};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// In opposition to a __typename field this field does retrieve data from a subgraph
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DataField @meta(module: "field/data", debug: false) @indexed(id_size: "u16") {
///   response_key: ResponseKey!
///   location: Location!
///   directives: [ExecutableDirective!]!
///   definition: FieldDefinition!
///   "Ordered by input value definition id"
///   arguments: [FieldArgument!]!
///   selection_set: SelectionSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DataFieldRecord {
    pub response_key: ResponseKey,
    pub location: Location,
    pub directive_ids: Vec<ExecutableDirectiveId>,
    pub definition_id: FieldDefinitionId,
    /// Ordered by input value definition id
    pub argument_ids: IdRange<FieldArgumentId>,
    pub selection_set_record: SelectionSetRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DataFieldId(std::num::NonZero<u16>);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub struct DataField<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: DataFieldId,
}

impl std::ops::Deref for DataField<'_> {
    type Target = DataFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> DataField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a DataFieldRecord {
        &self.ctx.operation[self.id]
    }
    pub fn directives(&self) -> impl Iter<Item = ExecutableDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.ctx)
    }
    pub fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
    /// Ordered by input value definition id
    pub fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.as_ref().argument_ids.walk(self.ctx)
    }
    pub fn selection_set(&self) -> SelectionSet<'a> {
        self.as_ref().selection_set_record.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for DataFieldId {
    type Walker<'w>
        = DataField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DataField {
            ctx: ctx.into(),
            id: self,
        }
    }
}

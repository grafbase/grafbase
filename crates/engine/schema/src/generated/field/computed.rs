//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{FieldDefinition, FieldDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ComputedObject @meta(module: "field/computed") {
///   subgraph: Subgraph!
///   fields: [ComputedField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ComputedObjectRecord {
    pub subgraph_id: SubgraphId,
    pub field_records: Vec<ComputedFieldRecord>,
}

#[derive(Clone, Copy)]
pub struct ComputedObject<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a ComputedObjectRecord,
}

impl std::ops::Deref for ComputedObject<'_> {
    type Target = ComputedObjectRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> ComputedObject<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ComputedObjectRecord {
        self.ref_
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn fields(&self) -> impl Iter<Item = ComputedField<'a>> + 'a {
        self.as_ref().field_records.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &ComputedObjectRecord {
    type Walker<'w>
        = ComputedObject<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ComputedObject {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for ComputedObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputedObject")
            .field("subgraph", &self.subgraph())
            .field("fields", &self.fields())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ComputedField @meta(module: "field/computed") {
///   from: FieldDefinition!
///   target: FieldDefinition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ComputedFieldRecord {
    pub from_id: FieldDefinitionId,
    pub target_id: FieldDefinitionId,
}

#[derive(Clone, Copy)]
pub struct ComputedField<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a ComputedFieldRecord,
}

impl std::ops::Deref for ComputedField<'_> {
    type Target = ComputedFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> ComputedField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ComputedFieldRecord {
        self.ref_
    }
    pub fn from(&self) -> FieldDefinition<'a> {
        self.from_id.walk(self.schema)
    }
    pub fn target(&self) -> FieldDefinition<'a> {
        self.target_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &ComputedFieldRecord {
    type Walker<'w>
        = ComputedField<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ComputedField {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for ComputedField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputedField")
            .field("from", &self.from())
            .field("target", &self.target())
            .finish()
    }
}

//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    DeriveMapping, DeriveMappingRecord,
    generated::{FieldDefinition, FieldDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DeriveDefinition @meta(module: "field/derive") @indexed(id_size: "u32") {
///   subgraph: Subgraph!
///   batch_field: FieldDefinition
///   mapping: DeriveMapping!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DeriveDefinitionRecord {
    pub subgraph_id: SubgraphId,
    pub batch_field_id: Option<FieldDefinitionId>,
    pub mapping_record: DeriveMappingRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DeriveDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct DeriveDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: DeriveDefinitionId,
}

impl std::ops::Deref for DeriveDefinition<'_> {
    type Target = DeriveDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> DeriveDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a DeriveDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn batch_field(&self) -> Option<FieldDefinition<'a>> {
        self.batch_field_id.walk(self.schema)
    }
    pub fn mapping(&self) -> DeriveMapping<'a> {
        self.as_ref().mapping_record.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for DeriveDefinitionId {
    type Walker<'w>
        = DeriveDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DeriveDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for DeriveDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeriveDefinition")
            .field("subgraph", &self.subgraph())
            .field("batch_field", &self.batch_field())
            .field("mapping", &self.mapping())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DeriveScalarAsField @meta(module: "field/derive") @copy {
///   field: FieldDefinition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct DeriveScalarAsFieldRecord {
    pub field_id: FieldDefinitionId,
}

#[derive(Clone, Copy)]
pub struct DeriveScalarAsField<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: DeriveScalarAsFieldRecord,
}

impl std::ops::Deref for DeriveScalarAsField<'_> {
    type Target = DeriveScalarAsFieldRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> DeriveScalarAsField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &DeriveScalarAsFieldRecord {
        &self.item
    }
    pub fn field(&self) -> FieldDefinition<'a> {
        self.field_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for DeriveScalarAsFieldRecord {
    type Walker<'w>
        = DeriveScalarAsField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DeriveScalarAsField {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for DeriveScalarAsField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeriveScalarAsField")
            .field("field", &self.field())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DeriveObject @meta(module: "field/derive") {
///   fields: [DeriveObjectField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DeriveObjectRecord {
    pub field_records: Vec<DeriveObjectFieldRecord>,
}

#[derive(Clone, Copy)]
pub struct DeriveObject<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a DeriveObjectRecord,
}

impl std::ops::Deref for DeriveObject<'_> {
    type Target = DeriveObjectRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> DeriveObject<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a DeriveObjectRecord {
        self.ref_
    }
    pub fn fields(&self) -> impl Iter<Item = DeriveObjectField<'a>> + 'a {
        self.as_ref().field_records.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &DeriveObjectRecord {
    type Walker<'w>
        = DeriveObject<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DeriveObject {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for DeriveObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeriveObject").field("fields", &self.fields()).finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DeriveObjectField @meta(module: "field/derive", derive: ["PartialEq", "Eq", "PartialOrd", "Ord"]) @copy {
///   from: FieldDefinition!
///   to: FieldDefinition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DeriveObjectFieldRecord {
    pub from_id: FieldDefinitionId,
    pub to_id: FieldDefinitionId,
}

#[derive(Clone, Copy)]
pub struct DeriveObjectField<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: DeriveObjectFieldRecord,
}

impl std::ops::Deref for DeriveObjectField<'_> {
    type Target = DeriveObjectFieldRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> DeriveObjectField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &DeriveObjectFieldRecord {
        &self.item
    }
    pub fn from(&self) -> FieldDefinition<'a> {
        self.from_id.walk(self.schema)
    }
    pub fn to(&self) -> FieldDefinition<'a> {
        self.to_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for DeriveObjectFieldRecord {
    type Walker<'w>
        = DeriveObjectField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DeriveObjectField {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for DeriveObjectField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeriveObjectField")
            .field("from", &self.from())
            .field("to", &self.to())
            .finish()
    }
}

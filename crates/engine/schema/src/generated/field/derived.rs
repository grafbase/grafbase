//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{EntityDefinition, EntityDefinitionId, FieldDefinition, FieldDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DerivedField @meta(module: "field/derived") @indexed(id_size: "u32") {
///   subgraph: Subgraph!
///   "Same as the parent entity of the derived FieldDefinition"
///   parent_entity: EntityDefinition!
///   mapping: [DerivedFieldMapping!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DerivedFieldRecord {
    pub subgraph_id: SubgraphId,
    /// Same as the parent entity of the derived FieldDefinition
    pub parent_entity_id: EntityDefinitionId,
    pub mapping_records: Vec<DerivedFieldMappingRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DerivedFieldId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct DerivedField<'a> {
    pub(crate) schema: &'a Schema,
    pub id: DerivedFieldId,
}

impl std::ops::Deref for DerivedField<'_> {
    type Target = DerivedFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> DerivedField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a DerivedFieldRecord {
        &self.schema[self.id]
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    /// Same as the parent entity of the derived FieldDefinition
    pub fn parent_entity(&self) -> EntityDefinition<'a> {
        self.parent_entity_id.walk(self.schema)
    }
    pub fn mapping(&self) -> impl Iter<Item = DerivedFieldMapping<'a>> + 'a {
        self.as_ref().mapping_records.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for DerivedFieldId {
    type Walker<'w>
        = DerivedField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DerivedField {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for DerivedField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedField")
            .field("subgraph", &self.subgraph())
            .field("parent_entity", &self.parent_entity())
            .field("mapping", &self.mapping())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DerivedFieldMapping @meta(module: "field/derived") @copy {
///   from: FieldDefinition!
///   to: FieldDefinition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct DerivedFieldMappingRecord {
    pub from_id: FieldDefinitionId,
    pub to_id: FieldDefinitionId,
}

#[derive(Clone, Copy)]
pub struct DerivedFieldMapping<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: DerivedFieldMappingRecord,
}

impl std::ops::Deref for DerivedFieldMapping<'_> {
    type Target = DerivedFieldMappingRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> DerivedFieldMapping<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &DerivedFieldMappingRecord {
        &self.item
    }
    pub fn from(&self) -> FieldDefinition<'a> {
        self.from_id.walk(self.schema)
    }
    pub fn to(&self) -> FieldDefinition<'a> {
        self.to_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for DerivedFieldMappingRecord {
    type Walker<'w>
        = DerivedFieldMapping<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DerivedFieldMapping {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for DerivedFieldMapping<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedFieldMapping")
            .field("from", &self.from())
            .field("to", &self.to())
            .finish()
    }
}

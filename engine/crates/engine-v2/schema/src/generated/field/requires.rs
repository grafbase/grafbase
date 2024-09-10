//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{Subgraph, SubgraphId},
    prelude::*,
    RequiredFieldSet, RequiredFieldSetId,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldRequires @meta(module: "field/requires") @copy {
///   subgraph: Subgraph!
///   field_set: RequiredFieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct FieldRequiresRecord {
    pub subgraph_id: SubgraphId,
    pub field_set_id: RequiredFieldSetId,
}

#[derive(Clone, Copy)]
pub struct FieldRequires<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: FieldRequiresRecord,
}

impl std::ops::Deref for FieldRequires<'_> {
    type Target = FieldRequiresRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> FieldRequires<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &FieldRequiresRecord {
        &self.item
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn field_set(&self) -> RequiredFieldSet<'a> {
        self.field_set_id.walk(self.schema)
    }
}

impl Walk<Schema> for FieldRequiresRecord {
    type Walker<'a> = FieldRequires<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        FieldRequires { schema, item: self }
    }
}

impl std::fmt::Debug for FieldRequires<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldRequires")
            .field("subgraph", &self.subgraph())
            .field("field_set", &self.field_set())
            .finish()
    }
}

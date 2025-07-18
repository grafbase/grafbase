//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    FieldSet, FieldSetRecord,
    generated::{Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldProvides @meta(module: "field/provides") {
///   subgraph: Subgraph!
///   field_set: FieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldProvidesRecord {
    pub subgraph_id: SubgraphId,
    pub field_set_record: FieldSetRecord,
}

#[derive(Clone, Copy)]
pub struct FieldProvides<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a FieldProvidesRecord,
}

impl std::ops::Deref for FieldProvides<'_> {
    type Target = FieldProvidesRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> FieldProvides<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldProvidesRecord {
        self.ref_
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn field_set(&self) -> FieldSet<'a> {
        self.as_ref().field_set_record.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &FieldProvidesRecord {
    type Walker<'w>
        = FieldProvides<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldProvides {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for FieldProvides<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldProvides")
            .field("subgraph", &self.subgraph())
            .field("field_set", &self.field_set())
            .finish()
    }
}

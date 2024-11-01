//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{Subgraph, SubgraphId},
    prelude::*,
    ProvidableFieldSet,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldProvides @meta(module: "field/provides") {
///   subgraph: Subgraph!
///   field_set: ProvidableFieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldProvidesRecord {
    pub subgraph_id: SubgraphId,
    pub field_set: ProvidableFieldSet,
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
}

impl<'a> Walk<&'a Schema> for &FieldProvidesRecord {
    type Walker<'w> = FieldProvides<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldProvides { schema, ref_: self }
    }
}

impl std::fmt::Debug for FieldProvides<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldProvides")
            .field("subgraph", &self.subgraph())
            .field("field_set", &self.field_set)
            .finish()
    }
}

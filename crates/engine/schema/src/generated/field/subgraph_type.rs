//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{Subgraph, SubgraphId, Type, TypeRecord},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SubgraphType @meta(module: "field/subgraph_type") @copy {
///   subgraph: Subgraph!
///   ty: Type!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SubgraphTypeRecord {
    pub subgraph_id: SubgraphId,
    pub ty_record: TypeRecord,
}

#[derive(Clone, Copy)]
pub struct SubgraphType<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: SubgraphTypeRecord,
}

impl std::ops::Deref for SubgraphType<'_> {
    type Target = SubgraphTypeRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> SubgraphType<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &SubgraphTypeRecord {
        &self.item
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn ty(&self) -> Type<'a> {
        self.ty_record.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for SubgraphTypeRecord {
    type Walker<'w>
        = SubgraphType<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SubgraphType {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for SubgraphType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphType")
            .field("subgraph", &self.subgraph())
            .field("ty", &self.ty())
            .finish()
    }
}

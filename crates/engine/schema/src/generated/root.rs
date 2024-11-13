//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ObjectDefinition, ObjectDefinitionId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type RootOperationTypes @meta(module: "root") {
///   query: ObjectDefinition!
///   mutation: ObjectDefinition
///   subscription: ObjectDefinition
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RootOperationTypesRecord {
    pub query_id: ObjectDefinitionId,
    pub mutation_id: Option<ObjectDefinitionId>,
    pub subscription_id: Option<ObjectDefinitionId>,
}

#[derive(Clone, Copy)]
pub struct RootOperationTypes<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a RootOperationTypesRecord,
}

impl std::ops::Deref for RootOperationTypes<'_> {
    type Target = RootOperationTypesRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> RootOperationTypes<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a RootOperationTypesRecord {
        self.ref_
    }
    pub fn query(&self) -> ObjectDefinition<'a> {
        self.query_id.walk(self.schema)
    }
    pub fn mutation(&self) -> Option<ObjectDefinition<'a>> {
        self.mutation_id.walk(self.schema)
    }
    pub fn subscription(&self) -> Option<ObjectDefinition<'a>> {
        self.subscription_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &RootOperationTypesRecord {
    type Walker<'w> = RootOperationTypes<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        RootOperationTypes {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for RootOperationTypes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootOperationTypes")
            .field("query", &self.query())
            .field("mutation", &self.mutation())
            .field("subscription", &self.subscription())
            .finish()
    }
}

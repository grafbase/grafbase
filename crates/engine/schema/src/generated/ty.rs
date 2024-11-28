//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{Definition, DefinitionId},
    prelude::*,
    Wrapping,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type Type @meta(module: "ty") @copy {
///   definition: Definition!
///   wrapping: Wrapping!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct TypeRecord {
    pub definition_id: DefinitionId,
    pub wrapping: Wrapping,
}

#[derive(Clone, Copy)]
pub struct Type<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: TypeRecord,
}

impl std::ops::Deref for Type<'_> {
    type Target = TypeRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> Type<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &TypeRecord {
        &self.item
    }
    pub fn definition(&self) -> Definition<'a> {
        self.definition_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for TypeRecord {
    type Walker<'w>
        = Type<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        Type {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type")
            .field("definition", &self.definition())
            .field("wrapping", &self.wrapping)
            .finish()
    }
}

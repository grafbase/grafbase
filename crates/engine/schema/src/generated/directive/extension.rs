//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    ExtensionDirectiveArgumentId, ExtensionDirectiveType, FieldSet, FieldSetRecord, StringId,
    generated::{Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ExtensionDirective @meta(module: "directive/extension") @indexed(id_size: "u32") {
///   subgraph: Subgraph!
///   extension_id: ExtensionId!
///   ty: ExtensionDirectiveType!
///   name: String!
///   argument_ids: [ExtensionDirectiveArgumentId!]!
///   requirements: FieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ExtensionDirectiveRecord {
    pub subgraph_id: SubgraphId,
    pub extension_id: ExtensionId,
    pub ty: ExtensionDirectiveType,
    pub name_id: StringId,
    pub argument_ids: IdRange<ExtensionDirectiveArgumentId>,
    pub requirements_record: FieldSetRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionDirectiveId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ExtensionDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ExtensionDirectiveId,
}

impl std::ops::Deref for ExtensionDirective<'_> {
    type Target = ExtensionDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ExtensionDirective<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ExtensionDirectiveRecord {
        &self.schema[self.id]
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn requirements(&self) -> FieldSet<'a> {
        self.as_ref().requirements_record.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ExtensionDirectiveId {
    type Walker<'w>
        = ExtensionDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ExtensionDirective {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ExtensionDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionDirective")
            .field("subgraph", &self.subgraph())
            .field("extension_id", &self.extension_id)
            .field("ty", &self.ty)
            .field("name", &self.name())
            .field("argument_ids", &self.argument_ids)
            .field("requirements", &self.requirements())
            .finish()
    }
}

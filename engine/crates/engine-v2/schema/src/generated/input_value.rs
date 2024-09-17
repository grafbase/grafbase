//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{Type, TypeRecord, TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    SchemaInputValue, SchemaInputValueId, StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type InputValueDefinition @meta(module: "input_value") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   ty: Type!
///   default_value: SchemaInputValue
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InputValueDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub ty_record: TypeRecord,
    pub default_value_id: Option<SchemaInputValueId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct InputValueDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: InputValueDefinitionId,
}

impl std::ops::Deref for InputValueDefinition<'_> {
    type Target = InputValueDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> InputValueDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a InputValueDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> InputValueDefinitionId {
        self.id
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn ty(&self) -> Type<'a> {
        self.ty_record.walk(self.schema)
    }
    pub fn default_value(&self) -> Option<SchemaInputValue<'a>> {
        self.default_value_id.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
}

impl Walk<Schema> for InputValueDefinitionId {
    type Walker<'a> = InputValueDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        InputValueDefinition { schema, id: self }
    }
}

impl std::fmt::Debug for InputValueDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputValueDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("ty", &self.ty().to_string())
            .field("default_value", &self.default_value())
            .field(
                "directives",
                &self.directives().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

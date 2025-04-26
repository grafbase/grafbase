//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    ArgumentValueInjection, StringId, ValueInjection,
    generated::{InputValueDefinition, InputValueDefinitionId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type KeyValueInjection @meta(module: "injection") @copy @indexed(id_size: "u32") {
///   key: String!
///   value: ValueInjection!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct KeyValueInjectionRecord {
    pub key_id: StringId,
    pub value: ValueInjection,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct KeyValueInjectionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct KeyValueInjection<'a> {
    pub(crate) schema: &'a Schema,
    pub id: KeyValueInjectionId,
}

impl std::ops::Deref for KeyValueInjection<'_> {
    type Target = KeyValueInjectionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> KeyValueInjection<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a KeyValueInjectionRecord {
        &self.schema[self.id]
    }
    pub fn key(&self) -> &'a str {
        self.key_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for KeyValueInjectionId {
    type Walker<'w>
        = KeyValueInjection<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        KeyValueInjection {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for KeyValueInjection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyValueInjection")
            .field("key", &self.key())
            .field("value", &self.value)
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ArgumentInjection @meta(module: "injection") @copy @indexed(id_size: "u32") {
///   definition: InputValueDefinition!
///   value: ArgumentValueInjection!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ArgumentInjectionRecord {
    pub definition_id: InputValueDefinitionId,
    pub value: ArgumentValueInjection,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ArgumentInjectionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ArgumentInjection<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ArgumentInjectionId,
}

impl std::ops::Deref for ArgumentInjection<'_> {
    type Target = ArgumentInjectionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ArgumentInjection<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ArgumentInjectionRecord {
        &self.schema[self.id]
    }
    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ArgumentInjectionId {
    type Walker<'w>
        = ArgumentInjection<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ArgumentInjection {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ArgumentInjection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArgumentInjection")
            .field("definition", &self.definition())
            .field("value", &self.value)
            .finish()
    }
}

//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
mod forward;
mod insert;
mod remove;
mod rename_duplicate;

use crate::{prelude::*, RegexId, StringId};
pub use forward::*;
pub use insert::*;
pub use remove::*;
pub use rename_duplicate::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union NameOrPattern @id @meta(module: "header_rule") @variants(names: ["Pattern", "Name"]) = Regex | String
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameOrPatternId {
    Name(StringId),
    Pattern(RegexId),
}

impl std::fmt::Debug for NameOrPatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameOrPatternId::Name(variant) => variant.fmt(f),
            NameOrPatternId::Pattern(variant) => variant.fmt(f),
        }
    }
}

impl From<StringId> for NameOrPatternId {
    fn from(value: StringId) -> Self {
        NameOrPatternId::Name(value)
    }
}
impl From<RegexId> for NameOrPatternId {
    fn from(value: RegexId) -> Self {
        NameOrPatternId::Pattern(value)
    }
}

#[derive(Clone, Copy)]
pub enum NameOrPattern<'a> {
    Name(&'a str),
    Pattern(&'a Regex),
}

impl std::fmt::Debug for NameOrPattern<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameOrPattern::Name(variant) => variant.fmt(f),
            NameOrPattern::Pattern(variant) => variant.fmt(f),
        }
    }
}

impl Walk<Schema> for NameOrPatternId {
    type Walker<'a> = NameOrPattern<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            NameOrPatternId::Name(id) => NameOrPattern::Name(&schema[id]),
            NameOrPatternId::Pattern(id) => NameOrPattern::Pattern(&schema[id]),
        }
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union HeaderRule
///   @meta(module: "header_rule")
///   @variants(remove_suffix: true)
///   @indexed(id_size: "u32", max_id: "MAX_ID", deduplicated: true) =
///   | ForwardHeaderRule
///   | InsertHeaderRule
///   | RemoveHeaderRule
///   | RenameDuplicateHeaderRule
/// ```
#[derive(serde::Serialize, serde::Deserialize)]
pub enum HeaderRuleRecord {
    Forward(ForwardHeaderRuleRecord),
    Insert(InsertHeaderRuleRecord),
    Remove(RemoveHeaderRuleRecord),
    RenameDuplicate(RenameDuplicateHeaderRuleRecord),
}

impl std::fmt::Debug for HeaderRuleRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderRuleRecord::Forward(variant) => variant.fmt(f),
            HeaderRuleRecord::Insert(variant) => variant.fmt(f),
            HeaderRuleRecord::Remove(variant) => variant.fmt(f),
            HeaderRuleRecord::RenameDuplicate(variant) => variant.fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct HeaderRuleId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct HeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: HeaderRuleId,
}

#[derive(Clone, Copy)]
pub enum HeaderRuleVariant<'a> {
    Forward(ForwardHeaderRule<'a>),
    Insert(InsertHeaderRule<'a>),
    Remove(RemoveHeaderRule<'a>),
    RenameDuplicate(RenameDuplicateHeaderRule<'a>),
}

impl std::fmt::Debug for HeaderRuleVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderRuleVariant::Forward(variant) => variant.fmt(f),
            HeaderRuleVariant::Insert(variant) => variant.fmt(f),
            HeaderRuleVariant::Remove(variant) => variant.fmt(f),
            HeaderRuleVariant::RenameDuplicate(variant) => variant.fmt(f),
        }
    }
}

impl std::ops::Deref for HeaderRule<'_> {
    type Target = HeaderRuleRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> HeaderRule<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a HeaderRuleRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> HeaderRuleId {
        self.id
    }
    pub fn variant(&self) -> HeaderRuleVariant<'a> {
        let schema = self.schema;
        match self.as_ref() {
            HeaderRuleRecord::Forward(item) => HeaderRuleVariant::Forward(item.walk(schema)),
            HeaderRuleRecord::Insert(item) => HeaderRuleVariant::Insert(item.walk(schema)),
            HeaderRuleRecord::Remove(item) => HeaderRuleVariant::Remove(item.walk(schema)),
            HeaderRuleRecord::RenameDuplicate(item) => HeaderRuleVariant::RenameDuplicate(item.walk(schema)),
        }
    }
}

impl Walk<Schema> for HeaderRuleId {
    type Walker<'a> = HeaderRule<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        HeaderRule { schema, id: self }
    }
}

impl std::fmt::Debug for HeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variant().fmt(f)
    }
}

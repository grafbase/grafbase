//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
mod forward;
mod insert;
mod remove;
mod rename_duplicate;

use crate::{RegexId, StringId, prelude::*};
pub use forward::*;
pub use insert::*;
pub use remove::*;
pub use rename_duplicate::*;
#[allow(unused_imports)]
use walker::{Iter, Walk};

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

impl NameOrPatternId {
    pub fn is_name(&self) -> bool {
        matches!(self, NameOrPatternId::Name(_))
    }
    pub fn as_name(&self) -> Option<StringId> {
        match self {
            NameOrPatternId::Name(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_pattern(&self) -> bool {
        matches!(self, NameOrPatternId::Pattern(_))
    }
    pub fn as_pattern(&self) -> Option<RegexId> {
        match self {
            NameOrPatternId::Pattern(id) => Some(*id),
            _ => None,
        }
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

impl<'a> Walk<&'a Schema> for NameOrPatternId {
    type Walker<'w>
        = NameOrPattern<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            NameOrPatternId::Name(id) => NameOrPattern::Name(&schema[id]),
            NameOrPatternId::Pattern(id) => NameOrPattern::Pattern(&schema[id]),
        }
    }
}

impl<'a> NameOrPattern<'a> {
    pub fn is_name(&self) -> bool {
        matches!(self, NameOrPattern::Name(_))
    }
    pub fn as_name(&self) -> Option<&'a str> {
        match self {
            NameOrPattern::Name(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_pattern(&self) -> bool {
        matches!(self, NameOrPattern::Pattern(_))
    }
    pub fn as_pattern(&self) -> Option<&'a Regex> {
        match self {
            NameOrPattern::Pattern(item) => Some(*item),
            _ => None,
        }
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union HeaderRule @meta(module: "header_rule") @variants(remove_suffix: true) @indexed(id_size: "u32") =
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

impl From<ForwardHeaderRuleRecord> for HeaderRuleRecord {
    fn from(value: ForwardHeaderRuleRecord) -> Self {
        HeaderRuleRecord::Forward(value)
    }
}
impl From<InsertHeaderRuleRecord> for HeaderRuleRecord {
    fn from(value: InsertHeaderRuleRecord) -> Self {
        HeaderRuleRecord::Insert(value)
    }
}
impl From<RemoveHeaderRuleRecord> for HeaderRuleRecord {
    fn from(value: RemoveHeaderRuleRecord) -> Self {
        HeaderRuleRecord::Remove(value)
    }
}
impl From<RenameDuplicateHeaderRuleRecord> for HeaderRuleRecord {
    fn from(value: RenameDuplicateHeaderRuleRecord) -> Self {
        HeaderRuleRecord::RenameDuplicate(value)
    }
}

impl HeaderRuleRecord {
    pub fn is_forward(&self) -> bool {
        matches!(self, HeaderRuleRecord::Forward(_))
    }
    pub fn as_forward(&self) -> Option<ForwardHeaderRuleRecord> {
        match self {
            HeaderRuleRecord::Forward(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_insert(&self) -> bool {
        matches!(self, HeaderRuleRecord::Insert(_))
    }
    pub fn as_insert(&self) -> Option<InsertHeaderRuleRecord> {
        match self {
            HeaderRuleRecord::Insert(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_remove(&self) -> bool {
        matches!(self, HeaderRuleRecord::Remove(_))
    }
    pub fn as_remove(&self) -> Option<RemoveHeaderRuleRecord> {
        match self {
            HeaderRuleRecord::Remove(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_rename_duplicate(&self) -> bool {
        matches!(self, HeaderRuleRecord::RenameDuplicate(_))
    }
    pub fn as_rename_duplicate(&self) -> Option<RenameDuplicateHeaderRuleRecord> {
        match self {
            HeaderRuleRecord::RenameDuplicate(item) => Some(*item),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct HeaderRuleId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct HeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub id: HeaderRuleId,
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
    pub fn variant(&self) -> HeaderRuleVariant<'a> {
        let schema = self.schema;
        match self.as_ref() {
            HeaderRuleRecord::Forward(item) => HeaderRuleVariant::Forward(item.walk(schema)),
            HeaderRuleRecord::Insert(item) => HeaderRuleVariant::Insert(item.walk(schema)),
            HeaderRuleRecord::Remove(item) => HeaderRuleVariant::Remove(item.walk(schema)),
            HeaderRuleRecord::RenameDuplicate(item) => HeaderRuleVariant::RenameDuplicate(item.walk(schema)),
        }
    }
    pub fn is_forward(&self) -> bool {
        matches!(self.variant(), HeaderRuleVariant::Forward(_))
    }
    pub fn as_forward(&self) -> Option<ForwardHeaderRule<'a>> {
        match self.variant() {
            HeaderRuleVariant::Forward(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_insert(&self) -> bool {
        matches!(self.variant(), HeaderRuleVariant::Insert(_))
    }
    pub fn as_insert(&self) -> Option<InsertHeaderRule<'a>> {
        match self.variant() {
            HeaderRuleVariant::Insert(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_remove(&self) -> bool {
        matches!(self.variant(), HeaderRuleVariant::Remove(_))
    }
    pub fn as_remove(&self) -> Option<RemoveHeaderRule<'a>> {
        match self.variant() {
            HeaderRuleVariant::Remove(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_rename_duplicate(&self) -> bool {
        matches!(self.variant(), HeaderRuleVariant::RenameDuplicate(_))
    }
    pub fn as_rename_duplicate(&self) -> Option<RenameDuplicateHeaderRule<'a>> {
        match self.variant() {
            HeaderRuleVariant::RenameDuplicate(item) => Some(item),
            _ => None,
        }
    }
}

impl<'a> Walk<&'a Schema> for HeaderRuleId {
    type Walker<'w>
        = HeaderRule<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        HeaderRule {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for HeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variant().fmt(f)
    }
}

//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    PartitionDataField, PartitionDataFieldId, QueryModifierRule, ResponseModifierRule,
    generated::{PartitionField, PartitionFieldId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryModifier @meta(module: "modifier", derive: ["Clone"]) {
///   rule: QueryModifierRule!
///   impacts_root_object: Boolean!
///   impacted_fields: [PartitionField!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct QueryModifierRecord {
    pub rule: QueryModifierRule,
    pub impacts_root_object: bool,
    pub impacted_field_ids: Vec<PartitionFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct QueryModifier<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) ref_: &'a QueryModifierRecord,
}

impl std::ops::Deref for QueryModifier<'_> {
    type Target = QueryModifierRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> QueryModifier<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a QueryModifierRecord {
        self.ref_
    }
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = PartitionField<'a>> + 'a {
        self.as_ref().impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for &QueryModifierRecord {
    type Walker<'w>
        = QueryModifier<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryModifier {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for QueryModifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryModifier")
            .field("rule", &self.rule)
            .field("impacts_root_object", &self.impacts_root_object)
            .field("impacted_fields", &self.impacted_fields())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseModifierDefinition @meta(module: "modifier") {
///   rule: ResponseModifierRule!
///   impacted_fields: [PartitionDataField!]! @vec
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierDefinitionRecord {
    pub rule: ResponseModifierRule,
    pub impacted_field_ids: Vec<PartitionDataFieldId>,
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifierDefinition<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) ref_: &'a ResponseModifierDefinitionRecord,
}

impl std::ops::Deref for ResponseModifierDefinition<'_> {
    type Target = ResponseModifierDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> ResponseModifierDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseModifierDefinitionRecord {
        self.ref_
    }
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = PartitionDataField<'a>> + 'a {
        self.as_ref().impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for &ResponseModifierDefinitionRecord {
    type Walker<'w>
        = ResponseModifierDefinition<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifierDefinition {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for ResponseModifierDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseModifierDefinition")
            .field("rule", &self.rule)
            .field("impacted_fields", &self.impacted_fields())
            .finish()
    }
}

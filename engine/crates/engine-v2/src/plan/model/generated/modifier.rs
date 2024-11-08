//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{generated::PlanField, prelude::*, PlanFieldRefId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryModifierDefinition @meta(module: "modifier") {
///   rule: QueryModifierRule!
///   impacts_root_object: Boolean!
///   impacted_fields: [PlanFieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifierDefinitionRecord {
    pub rule: QueryModifierRule,
    pub impacts_root_object: bool,
    pub impacted_field_ids: IdRange<PlanFieldRefId>,
}

#[derive(Clone, Copy)]
pub(crate) struct QueryModifierDefinition<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) ref_: &'a QueryModifierDefinitionRecord,
}

impl std::ops::Deref for QueryModifierDefinition<'_> {
    type Target = QueryModifierDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> QueryModifierDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a QueryModifierDefinitionRecord {
        self.ref_
    }
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = PlanField<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for &QueryModifierDefinitionRecord {
    type Walker<'w> = QueryModifierDefinition<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryModifierDefinition { ctx, ref_: self }
    }
}

impl std::fmt::Debug for QueryModifierDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryModifierDefinition")
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
///   impacted_fields: [PlanFieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierDefinitionRecord {
    pub rule: ResponseModifierRule,
    pub impacted_field_ids: IdRange<PlanFieldRefId>,
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifierDefinition<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) ref_: &'a ResponseModifierDefinitionRecord,
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
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = PlanField<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for &ResponseModifierDefinitionRecord {
    type Walker<'w> = ResponseModifierDefinition<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifierDefinition { ctx, ref_: self }
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

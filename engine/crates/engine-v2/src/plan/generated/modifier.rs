//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::{generated::Field, prelude::*, FieldRefId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryModifier @meta(module: "modifier") {
///   rule: QueryModifierRule!
///   impacted_fields: [FieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifierRecord {
    pub rule: QueryModifierRule,
    pub impacted_field_ids: IdRange<FieldRefId>,
}

#[derive(Clone, Copy)]
pub(crate) struct QueryModifier<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) ref_: &'a QueryModifierRecord,
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
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for &QueryModifierRecord {
    type Walker<'w> = QueryModifier<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryModifier { ctx, ref_: self }
    }
}

impl std::fmt::Debug for QueryModifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryModifier")
            .field("rule", &self.rule)
            .field("impacted_fields", &self.impacted_fields())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseModifier @meta(module: "modifier") {
///   rule: ResponseModifierRule!
///   impacted_fields: [FieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierRecord {
    pub rule: ResponseModifierRule,
    pub impacted_field_ids: IdRange<FieldRefId>,
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifier<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) ref_: &'a ResponseModifierRecord,
}

impl std::ops::Deref for ResponseModifier<'_> {
    type Target = ResponseModifierRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> ResponseModifier<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseModifierRecord {
        self.ref_
    }
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for &ResponseModifierRecord {
    type Walker<'w> = ResponseModifier<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifier { ctx, ref_: self }
    }
}

impl std::fmt::Debug for ResponseModifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseModifier")
            .field("rule", &self.rule)
            .field("impacted_fields", &self.impacted_fields())
            .finish()
    }
}

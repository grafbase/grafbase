//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/solved_operation.graphql
use crate::operation::solve::model::{
    generated::{DataField, Field},
    prelude::*,
    DataFieldRefId, FieldRefId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryModifierDefinition @meta(module: "modifier") {
///   rule: QueryModifierRule!
///   impacts_root_object: Boolean!
///   impacted_fields: [FieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifierDefinitionRecord {
    pub rule: QueryModifierRule,
    pub impacts_root_object: bool,
    pub impacted_field_ids: IdRange<FieldRefId>,
}

#[derive(Clone, Copy)]
pub(crate) struct QueryModifierDefinition<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
    pub(in crate::operation::solve::model) ref_: &'a QueryModifierDefinitionRecord,
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
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<SolvedOperationContext<'a>> for &QueryModifierDefinitionRecord {
    type Walker<'w> = QueryModifierDefinition<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryModifierDefinition {
            ctx: ctx.into(),
            ref_: self,
        }
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
/// type ResponseModifierDefinition @meta(module: "modifier") @indexed(id_size: "u16") {
///   rule: ResponseModifierRule!
///   impacted_fields: [DataFieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierDefinitionRecord {
    pub rule: ResponseModifierRule,
    pub impacted_field_ids: IdRange<DataFieldRefId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ResponseModifierDefinitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifierDefinition<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
    pub(crate) id: ResponseModifierDefinitionId,
}

impl std::ops::Deref for ResponseModifierDefinition<'_> {
    type Target = ResponseModifierDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> ResponseModifierDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseModifierDefinitionRecord {
        &self.ctx.operation[self.id]
    }
    pub(crate) fn impacted_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.impacted_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<SolvedOperationContext<'a>> for ResponseModifierDefinitionId {
    type Walker<'w> = ResponseModifierDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifierDefinition {
            ctx: ctx.into(),
            id: self,
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

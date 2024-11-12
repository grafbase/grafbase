//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_solution.graphql
use crate::plan::solver::model::prelude::*;
use schema::{CompositeType, CompositeTypeId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseObjectSetDefinition @meta(module: "response_object_set") @indexed(id_size: "u16", deduplicated: true) {
///   ty: CompositeType!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseObjectSetDefinitionRecord {
    pub ty_id: CompositeTypeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ResponseObjectSetDefinitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct ResponseObjectSetDefinition<'a> {
    pub(in crate::plan::solver::model) ctx: OperationSolutionContext<'a>,
    pub(crate) id: ResponseObjectSetDefinitionId,
}

impl std::ops::Deref for ResponseObjectSetDefinition<'_> {
    type Target = ResponseObjectSetDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> ResponseObjectSetDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseObjectSetDefinitionRecord {
        &self.ctx.operation_solution[self.id]
    }
    pub(crate) fn ty(&self) -> CompositeType<'a> {
        self.ty_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationSolutionContext<'a>> for ResponseObjectSetDefinitionId {
    type Walker<'w> = ResponseObjectSetDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationSolutionContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseObjectSetDefinition {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ResponseObjectSetDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseObjectSetDefinition")
            .field("ty", &self.ty())
            .finish()
    }
}

//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::operation::plan::model::prelude::*;
use schema::{CompositeType, CompositeTypeId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseModifierTarget @meta(module: "response_modifier/target") {
///   set_id: ResponseObjectSetDefinitionId!
///   ty: CompositeType!
///   key: SafeResponseKey!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierTargetRecord {
    pub set_id: ResponseObjectSetDefinitionId,
    pub ty_id: CompositeTypeId,
    pub key: SafeResponseKey,
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifierTarget<'a> {
    pub(in crate::operation::plan::model) ctx: OperationPlanContext<'a>,
    pub(in crate::operation::plan::model) ref_: &'a ResponseModifierTargetRecord,
}

impl std::ops::Deref for ResponseModifierTarget<'_> {
    type Target = ResponseModifierTargetRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> ResponseModifierTarget<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseModifierTargetRecord {
        self.ref_
    }
    pub(crate) fn ty(&self) -> CompositeType<'a> {
        self.ty_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationPlanContext<'a>> for &ResponseModifierTargetRecord {
    type Walker<'w> = ResponseModifierTarget<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifierTarget {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for ResponseModifierTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseModifierTarget")
            .field("set_id", &self.set_id)
            .field("ty", &self.ty())
            .field("key", &self.key)
            .finish()
    }
}

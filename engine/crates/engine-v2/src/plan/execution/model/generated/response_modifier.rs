//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
mod target;

use crate::plan::execution::model::{
    generated::{Executable, ExecutableId},
    prelude::*,
};
use crate::plan::solver::{ResponseModifierDefinition, ResponseModifierDefinitionId};
pub(crate) use target::*;
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseModifier @indexed(id_size: "u16") @meta(module: "response_modifier") {
///   definition: ResponseModifierDefinition!
///   sorted_targets: [ResponseModifierTarget!]!
///   parent_count: usize!
///   children: [Executable!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifierRecord {
    pub definition_id: ResponseModifierDefinitionId,
    pub sorted_target_records: Vec<ResponseModifierTargetRecord>,
    pub parent_count: usize,
    pub children_ids: Vec<ExecutableId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ResponseModifierId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct ResponseModifier<'a> {
    pub(in crate::plan::execution::model) ctx: OperationPlanContext<'a>,
    pub(crate) id: ResponseModifierId,
}

impl std::ops::Deref for ResponseModifier<'_> {
    type Target = ResponseModifierRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> ResponseModifier<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseModifierRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn definition(&self) -> ResponseModifierDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
    pub(crate) fn sorted_targets(&self) -> impl Iter<Item = ResponseModifierTarget<'a>> + 'a {
        self.as_ref().sorted_target_records.walk(self.ctx)
    }
    pub(crate) fn children(&self) -> impl Iter<Item = Executable<'a>> + 'a {
        self.as_ref().children_ids.walk(self.ctx)
    }
}

impl<'a> Walk<OperationPlanContext<'a>> for ResponseModifierId {
    type Walker<'w> = ResponseModifier<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseModifier {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ResponseModifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseModifier")
            .field("definition", &self.definition())
            .field("sorted_targets", &self.sorted_targets())
            .field("parent_count", &self.parent_count)
            .field("children", &self.children())
            .finish()
    }
}

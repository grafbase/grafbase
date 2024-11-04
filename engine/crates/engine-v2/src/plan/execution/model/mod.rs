#![allow(unused)]
mod field;
mod plan;
mod selection_set;

use std::sync::Arc;

use schema::{EntityDefinitionId, RequiredFieldSetRecord, Schema};

use crate::{
    operation::{ResponseModifierRule, Variables},
    plan::{OperationPlan, PlanId, ResponseObjectSetDefinitionId},
    resolver::Resolver,
    response::{ResponseKey, ResponseViewSelectionSet, ResponseViews},
};

use super::QueryModifications;

pub(crate) use field::*;
pub(crate) use plan::*;
pub(crate) use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct QueryContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) operation_plan: &'a OperationPlan,
    pub(super) query_modifications: &'a QueryModifications,
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct ExecutionPlan {
    pub query_modifications: QueryModifications,
    #[indexed_by(PlanResolverId)]
    pub plan_resolvers: Vec<PlanResolver>,
    #[indexed_by(ResponseModifierId)]
    pub response_modifiers: Vec<ResponseModifier>,
}

#[derive(Clone, Copy)]
pub(crate) enum ExecutableId {
    PlanResolver(PlanResolverId),
    ResponseModifier(ResponseModifierId),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PlanResolverId(std::num::NonZero<u16>);

pub(crate) struct PlanResolver {
    pub plan_id: PlanId,
    pub requires: RequiredFieldSetRecord,
    pub resolver: Resolver,
    pub parent_count: usize,
    pub children: Vec<ExecutableId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ResponseModifierId(std::num::NonZero<u16>);

// Modifies the response based on a given rule
pub(crate) struct ResponseModifier {
    pub rule: ResponseModifierRule,
    /// Which object & fields are impacted
    /// sorted by natural order
    pub on: Vec<(ResponseObjectSetDefinitionId, Option<EntityDefinitionId>, ResponseKey)>,
    /// What fields the hook requires
    pub requires: ResponseViewSelectionSet,
    /// Dependency count
    pub parent_count: usize,
    /// Dependents
    pub children: Vec<ExecutableId>,
}

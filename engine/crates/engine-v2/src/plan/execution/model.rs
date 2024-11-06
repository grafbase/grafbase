#![allow(unused)]
use schema::EntityDefinitionId;

use crate::{
    operation::ResponseModifierRule,
    plan::{PlanId, ResponseObjectSetDefinitionId},
    resolver::Resolver,
    response::{ResponseKey, ResponseViewSelectionSet, ResponseViews},
};

use super::QueryModifications;

#[derive(id_derives::IndexedFields)]
pub(crate) struct ExecutionPlan {
    pub(crate) query_modifications: QueryModifications,
    pub(crate) response_views: ResponseViews,
    #[indexed_by(PlanResolverId)]
    pub(crate) plan_resolvers: Vec<PlanResolver>,
    #[indexed_by(ResponseModifierId)]
    pub(crate) response_modifiers: Vec<ResponseModifier>,
}

pub(crate) enum ExecutableId {
    PlanResolver(PlanResolverId),
    ResponseModifier(ResponseModifierId),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PlanResolverId(std::num::NonZero<u16>);

pub(crate) struct PlanResolver {
    pub plan_id: PlanId,
    pub requires: ResponseViewSelectionSet,
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

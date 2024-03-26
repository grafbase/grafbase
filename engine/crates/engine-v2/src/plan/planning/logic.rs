use schema::{FieldDefinitionId, ProvidableFieldSet, ResolverWalker};

use crate::plan::PlanId;

/// Defines whether a field can be provided or not for a given resolver.
#[derive(Debug, Clone)]
pub(super) enum PlanningLogic<'schema> {
    /// Having a resolver in the same group or having no resolver at all.
    SameSubgrah {
        plan_id: PlanId,
        resolver: ResolverWalker<'schema>,
        providable: ProvidableFieldSet,
    },
    /// Only an explicitly providable (@provide) field can be attributed.
    OnlyProvidable {
        plan_id: PlanId,
        resolver: ResolverWalker<'schema>,
        providable: ProvidableFieldSet,
    },
}

impl std::fmt::Display for PlanningLogic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlanningLogic{}", usize::from(self.plan_id()))
    }
}

impl<'schema> PlanningLogic<'schema> {
    pub(super) fn new(plan_id: PlanId, resolver: ResolverWalker<'schema>) -> Self {
        PlanningLogic::SameSubgrah {
            plan_id,
            resolver,
            providable: Default::default(),
        }
    }

    pub(super) fn is_providable(&self, field_id: FieldDefinitionId) -> bool {
        match self {
            PlanningLogic::SameSubgrah {
                resolver, providable, ..
            } => resolver.can_provide(field_id) || providable.contains(field_id),
            PlanningLogic::OnlyProvidable { providable, .. } => providable.contains(field_id),
        }
    }

    pub(super) fn child(&self, field_id: FieldDefinitionId) -> Self {
        match self {
            PlanningLogic::SameSubgrah {
                plan_id,
                resolver,
                providable,
            } => {
                let subgraph_id = resolver.subgraph_id();
                let providable = ProvidableFieldSet::union_opt(
                    providable.get(field_id).map(|s| &s.subselection),
                    Some(resolver.walk(field_id).provides(subgraph_id)),
                );
                if resolver.can_provide(field_id) {
                    PlanningLogic::SameSubgrah {
                        plan_id: *plan_id,
                        resolver: *resolver,
                        providable,
                    }
                } else {
                    PlanningLogic::OnlyProvidable {
                        plan_id: *plan_id,
                        resolver: *resolver,
                        providable,
                    }
                }
            }
            PlanningLogic::OnlyProvidable {
                resolver,
                providable,
                plan_id,
            } => PlanningLogic::OnlyProvidable {
                plan_id: *plan_id,
                resolver: *resolver,
                providable: providable
                    .get(field_id)
                    .map(|field| field.subselection.clone())
                    .unwrap_or_default(),
            },
        }
    }

    pub(super) fn resolver(&self) -> &ResolverWalker<'schema> {
        match self {
            PlanningLogic::SameSubgrah { resolver, .. } => resolver,
            PlanningLogic::OnlyProvidable { resolver, .. } => resolver,
        }
    }

    pub(super) fn plan_id(&self) -> PlanId {
        match self {
            PlanningLogic::SameSubgrah { plan_id, .. } => *plan_id,
            PlanningLogic::OnlyProvidable { plan_id, .. } => *plan_id,
        }
    }
}

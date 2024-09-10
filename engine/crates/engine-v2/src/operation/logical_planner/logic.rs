use schema::{FieldDefinitionId, ProvidableFieldSet, ResolverDefinition, Schema};

use crate::operation::LogicalPlanId;

/// Defines whether a field can be provided or not for a given resolver and
/// thus be added to the solved field set.
#[derive(Debug, Clone)]
pub(super) enum PlanningLogic<'schema> {
    /// Having a resolver in the same group or having no resolver at all.
    SameSubgrah {
        id: LogicalPlanId,
        schema: &'schema Schema,
        resolver: ResolverDefinition<'schema>,
        providable: ProvidableFieldSet,
    },
    /// Only an explicitly providable (@provide) field can be provided.
    OnlyProvidable {
        id: LogicalPlanId,
        schema: &'schema Schema,
        resolver: ResolverDefinition<'schema>,
        providable: ProvidableFieldSet,
    },
}

impl std::fmt::Display for PlanningLogic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlanningLogic#{}", usize::from(self.id()))
    }
}

impl<'schema> PlanningLogic<'schema> {
    pub(super) fn new(id: LogicalPlanId, schema: &'schema Schema, resolver: ResolverDefinition<'schema>) -> Self {
        PlanningLogic::SameSubgrah {
            id,
            schema,
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
                id,
                schema,
                resolver,
                providable,
            } => {
                let subgraph_id = resolver.subgraph_id();
                let providable = ProvidableFieldSet::union_opt(
                    providable.get(field_id).map(|s| &s.subselection),
                    Some(schema.walk(field_id).provides_for_subgraph(subgraph_id)),
                );
                if resolver.can_provide(field_id) {
                    PlanningLogic::SameSubgrah {
                        id: *id,
                        schema,
                        resolver: *resolver,
                        providable,
                    }
                } else {
                    PlanningLogic::OnlyProvidable {
                        id: *id,
                        schema,
                        resolver: *resolver,
                        providable,
                    }
                }
            }
            PlanningLogic::OnlyProvidable {
                resolver,
                schema,
                providable,
                id,
            } => PlanningLogic::OnlyProvidable {
                id: *id,
                schema,
                resolver: *resolver,
                providable: providable
                    .get(field_id)
                    .map(|field| field.subselection.clone())
                    .unwrap_or_default(),
            },
        }
    }

    pub(super) fn resolver(&self) -> &ResolverDefinition<'schema> {
        match self {
            PlanningLogic::SameSubgrah { resolver, .. } => resolver,
            PlanningLogic::OnlyProvidable { resolver, .. } => resolver,
        }
    }

    pub(super) fn id(&self) -> LogicalPlanId {
        match self {
            PlanningLogic::SameSubgrah { id, .. } => *id,
            PlanningLogic::OnlyProvidable { id, .. } => *id,
        }
    }
}

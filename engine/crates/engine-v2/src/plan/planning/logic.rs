use schema::{FieldId, FieldSet, ResolverWalker};

use crate::plan::PlanId;

/// Defines whether a field can be provided or not for a given resolver. Initially all fields with
/// no resolver or a compatible one (same subgraph typically) can be planned. Once we hit a
/// different resolver only those marked as `@providable` will be.
#[derive(Debug, Clone)]
pub(super) enum PlanningLogic<'schema> {
    /// Having a resolver in the same group or having no resolver at all.
    CompatibleResolver {
        plan_id: PlanId,
        resolver: ResolverWalker<'schema>,
        providable: FieldSet,
    },
    /// Only an explicitly providable (@provide) field can be attributed. This is an optimization
    /// overriding the CompatibleResolver logic
    OnlyProvidable {
        plan_id: PlanId,
        resolver: ResolverWalker<'schema>,
        providable: FieldSet,
    },
}

impl<'schema> PlanningLogic<'schema> {
    pub(super) fn is_providable(&self, field_id: FieldId) -> bool {
        match self {
            PlanningLogic::CompatibleResolver {
                resolver, providable, ..
            } => providable.get(field_id).is_some() || resolver.can_provide(field_id),
            PlanningLogic::OnlyProvidable { providable, .. } => providable.get(field_id).is_some(),
        }
    }

    // extra fields don't have a key, I'm not entirely whether that makes sense, maybe we should
    // just use ResponsePath. Not sure.
    pub(super) fn child(&self, field_id: FieldId) -> Self {
        match self {
            PlanningLogic::CompatibleResolver {
                resolver,
                providable,
                plan_id,
            } => {
                let providable = FieldSet::merge_opt(
                    providable.get(field_id).map(|s| &s.subselection),
                    resolver.walk(field_id).provides_for(resolver).as_deref(),
                );
                if resolver.can_provide(field_id) {
                    PlanningLogic::CompatibleResolver {
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
                    .map(|s| &s.subselection)
                    .cloned()
                    .unwrap_or_default(),
            },
        }
    }

    pub(super) fn resolver(&self) -> &ResolverWalker<'schema> {
        match self {
            PlanningLogic::CompatibleResolver { resolver, .. } => resolver,
            PlanningLogic::OnlyProvidable { resolver, .. } => resolver,
        }
    }

    pub(super) fn plan_id(&self) -> PlanId {
        match self {
            PlanningLogic::CompatibleResolver { plan_id, .. } => *plan_id,
            PlanningLogic::OnlyProvidable { plan_id, .. } => *plan_id,
        }
    }
}

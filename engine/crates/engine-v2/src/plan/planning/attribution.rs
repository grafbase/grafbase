use schema::{FieldId, FieldSet, ResolverWalker};

use crate::plan::PlanId;

#[derive(Debug, Clone)]
pub(super) enum AttributionLogic<'schema> {
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

impl<'schema> AttributionLogic<'schema> {
    pub(super) fn is_providable(&self, field_id: FieldId) -> bool {
        match self {
            AttributionLogic::CompatibleResolver {
                resolver, providable, ..
            } => providable.get(field_id).is_some() || resolver.can_provide(field_id),
            AttributionLogic::OnlyProvidable { providable, .. } => providable.get(field_id).is_some(),
        }
    }

    // extra fields don't have a key, I'm not entirely whether that makes sense, maybe we should
    // just use ResponsePath. Not sure.
    pub(super) fn child(&self, field_id: FieldId) -> Self {
        match self {
            AttributionLogic::CompatibleResolver {
                resolver,
                providable,
                plan_id,
            } => {
                let providable = FieldSet::merge_opt(
                    providable.get(field_id).map(|s| &s.subselection),
                    resolver.walk(field_id).provides_for(resolver).as_deref(),
                );
                if resolver.can_provide(field_id) {
                    AttributionLogic::CompatibleResolver {
                        plan_id: *plan_id,
                        resolver: *resolver,
                        providable,
                    }
                } else {
                    AttributionLogic::OnlyProvidable {
                        plan_id: *plan_id,
                        resolver: *resolver,
                        providable,
                    }
                }
            }
            AttributionLogic::OnlyProvidable {
                resolver,
                providable,
                plan_id,
            } => AttributionLogic::OnlyProvidable {
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
            AttributionLogic::CompatibleResolver { resolver, .. } => resolver,
            AttributionLogic::OnlyProvidable { resolver, .. } => resolver,
        }
    }

    pub(super) fn plan_id(&self) -> PlanId {
        match self {
            AttributionLogic::CompatibleResolver { plan_id, .. } => *plan_id,
            AttributionLogic::OnlyProvidable { plan_id, .. } => *plan_id,
        }
    }
}

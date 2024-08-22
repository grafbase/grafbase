use std::collections::HashSet;

use schema::{RequiredScopeSetIndex, RequiredScopesId, RequiredScopesWalker, TypeSystemDirectivesWalker};

use super::{LogicalPlanId, OperationPlan, OperationWalker, PlanWalker, SelectionSetWalker};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheScopeId(u16);

impl<'a, Item, SchemaItem> PlanWalker<'a, Item, SchemaItem> {
    /// The cache scopes that should be applied to the current plans cache
    pub fn cache_scopes(self) -> CacheScopes<'a> {
        CacheScopes {
            walker: self.walk_with((), ()),
            current_index: self
                .operation
                .logical_plan_cache_scopes
                .binary_search_by(|(plan_id, _)| plan_id.cmp(&self.logical_plan_id))
                .unwrap_or(self.operation.logical_plan_cache_scopes.len()),
        }
    }
}

pub struct CacheScopes<'a> {
    walker: PlanWalker<'a>,
    current_index: usize,
}

impl<'a> Iterator for CacheScopes<'a> {
    type Item = CacheScope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self
            .walker
            .operation
            .logical_plan_cache_scopes
            .get(self.current_index)?;

        if current.0 != self.walker.logical_plan_id {
            return None;
        }

        self.current_index += 1;

        Some(match self.walker.operation.cache_scopes[current.1 .0 as usize] {
            CacheScopeRecord::Authenticated => CacheScope::Authenticated,
            CacheScopeRecord::RequiresScopes(required_scopes_id) => CacheScope::RequiresScopes(RequiredScopeSet {
                walker: self.walker.schema_walker.walk(required_scopes_id),
                scope_set: self
                    .walker
                    .query_modifications
                    .selected_scope_set(required_scopes_id)
                    .expect("a scope set to be selected"),
            }),
        })
    }
}

pub enum CacheScope<'a> {
    Authenticated,
    RequiresScopes(RequiredScopeSet<'a>),
}

pub struct RequiredScopeSet<'a> {
    walker: RequiredScopesWalker<'a>,
    scope_set: RequiredScopeSetIndex,
}

impl<'a> RequiredScopeSet<'a> {
    pub fn scopes(&self) -> impl ExactSizeIterator<Item = &'a str> {
        self.walker.scopes(self.scope_set)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub(super) enum CacheScopeRecord {
    Authenticated,
    RequiresScopes(RequiredScopesId),
}

pub(super) fn calculate_cache_scopes(
    operation: OperationWalker<'_>,
    operation_plan: &OperationPlan,
) -> (Vec<(LogicalPlanId, CacheScopeId)>, Vec<CacheScopeRecord>) {
    let mut builder = CacheScopeBuilder::default();

    calculate_cache_scopes_for_selection_set(operation.selection_set(), None, operation_plan, &mut builder);

    let CacheScopeBuilder {
        mut plan_id_to_cache_scope_id,
        cache_scopes,
        plans_handled: _,
        scope_stack: _,
    } = builder;

    plan_id_to_cache_scope_id.sort_by_key(|(plan_id, _)| *plan_id);

    (plan_id_to_cache_scope_id, cache_scopes)
}

fn calculate_cache_scopes_for_selection_set(
    selection_set: SelectionSetWalker<'_>,
    parent_plan_id: Option<LogicalPlanId>,
    operation_plan: &OperationPlan,
    builder: &mut CacheScopeBuilder,
) {
    let mut last_parent_entity_id = None;
    let mut parent_entity_scopes_added = 0;

    for field in selection_set.fields() {
        let Some(definition) = field.definition() else { continue };

        // This takes advantage of fields being ordered by parent entity Id
        let parent_entity_id = definition.parent_entity().id();
        if Some(parent_entity_id) != last_parent_entity_id {
            last_parent_entity_id = Some(parent_entity_id);
            builder.pop_scopes(parent_entity_scopes_added);
            parent_entity_scopes_added = add_directive_scopes(definition.parent_entity().directives(), builder)
        }

        let scopes_added = add_directive_scopes(definition.directives(), builder);

        let current_plan_id = operation_plan.plan_id_for_field(field.item);
        if parent_plan_id != Some(current_plan_id) {
            builder.link_scopes_to_plan(current_plan_id)
        }

        if let Some(nested_selection_set) = field.selection_set() {
            calculate_cache_scopes_for_selection_set(
                nested_selection_set,
                Some(current_plan_id),
                operation_plan,
                builder,
            );
        }

        builder.pop_scopes(scopes_added);
    }

    builder.pop_scopes(parent_entity_scopes_added);
}

fn add_directive_scopes(directives: TypeSystemDirectivesWalker<'_>, builder: &mut CacheScopeBuilder) -> usize {
    let mut scopes_added = 0;
    for directive in directives.as_ref().iter() {
        match directive {
            schema::TypeSystemDirective::Deprecated(_)
            | schema::TypeSystemDirective::CacheControl(_)
            | schema::TypeSystemDirective::Authorized(_) => {}

            schema::TypeSystemDirective::Authenticated => {
                scopes_added += 1;
                builder.insert_scope(CacheScopeRecord::Authenticated);
            }
            schema::TypeSystemDirective::RequiresScopes(requires_scope_id) => {
                scopes_added += 1;
                builder.insert_scope(CacheScopeRecord::RequiresScopes(*requires_scope_id));
            }
        }
    }
    scopes_added
}

#[derive(Default)]
struct CacheScopeBuilder {
    plan_id_to_cache_scope_id: Vec<(LogicalPlanId, CacheScopeId)>,
    cache_scopes: Vec<CacheScopeRecord>,

    scope_stack: Vec<CacheScopeId>,
    plans_handled: HashSet<LogicalPlanId>,
}

impl CacheScopeBuilder {
    fn pop_scopes(&mut self, count: usize) {
        self.scope_stack.truncate(self.scope_stack.len() - count);
    }

    fn insert_scope(&mut self, record: CacheScopeRecord) {
        let id = CacheScopeId(self.cache_scopes.len() as u16);
        self.cache_scopes.push(record);
        self.scope_stack.push(id);
    }

    fn link_scopes_to_plan(&mut self, plan: LogicalPlanId) {
        if self.plans_handled.contains(&plan) {
            return;
        }
        self.plans_handled.insert(plan);
        self.plan_id_to_cache_scope_id
            .extend(self.scope_stack.iter().map(|scope_id| (plan, *scope_id)))
    }
}

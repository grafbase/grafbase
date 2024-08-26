mod logic;
mod selection_set;

use std::borrow::Cow;

use engine_parser::types::OperationType;
use id_derives::IndexedFields;
use id_newtypes::{BitSet, IdToMany};
use itertools::Itertools;
use schema::{EntityId, FieldDefinitionId, RequiredFieldId, RequiredFieldSet, ResolverDefinitionId, Schema};
use tracing::instrument;

use crate::{
    operation::{
        FieldId, LogicalPlan, LogicalPlanId, Operation, OperationWalker, QueryPath, ResponseModifierRule,
        SelectionSetId, SolvedRequiredFieldSet,
    },
    response::{ErrorCode, GraphqlError},
};
use logic::*;
use selection_set::*;

use super::OperationPlan;

#[derive(Debug, thiserror::Error)]
pub(crate) enum LogicalPlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField {
        missing: Vec<String>,
        query_path: Vec<String>,
    },
}

impl From<LogicalPlanningError> for GraphqlError {
    fn from(error: LogicalPlanningError) -> Self {
        let message = error.to_string();
        let query_path = match error {
            LogicalPlanningError::CouldNotPlanAnyField { query_path, .. } => query_path
                .into_iter()
                .map(serde_json::Value::String)
                .collect::<Vec<_>>(),
        };

        GraphqlError::new(message, ErrorCode::OperationPlanningError).with_extension("queryPath", query_path)
    }
}

pub(super) type LogicalPlanningResult<T> = Result<T, LogicalPlanningError>;

#[derive(IndexedFields)]
pub(super) struct LogicalPlanner<'a> {
    schema: &'a Schema,
    operation: &'a mut Operation,
    #[indexed_by(FieldId)]
    field_to_logical_plan_id: Vec<Option<LogicalPlanId>>,
    field_to_solved_requirement: Vec<Option<RequiredFieldId>>,
    #[indexed_by(LogicalPlanId)]
    logical_plans: Vec<LogicalPlan>,
    selection_set_to_objects_must_be_tracked: BitSet<SelectionSetId>,
    mutation_fields_plan_order: Vec<LogicalPlanId>,
    // May have duplicates, parent may be equal to child (if we, as the supergraph, need the dependencies)
    // (parent, child)
    dependents_builder: Vec<(LogicalPlanId, LogicalPlanId)>,
    solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParentToChildEdge {
    pub parent: LogicalPlanId,
    pub child: LogicalPlanId,
}

impl<'a> LogicalPlanner<'a> {
    pub(super) fn new(schema: &'a Schema, operation: &'a mut Operation) -> Self {
        Self {
            schema,
            field_to_logical_plan_id: vec![None; operation.fields.len()],
            field_to_solved_requirement: vec![None; operation.fields.len()],
            selection_set_to_objects_must_be_tracked: BitSet::init_with(false, operation.selection_sets.len()),
            operation,
            logical_plans: Vec::new(),
            solved_requirements: Vec::new(),
            dependents_builder: Vec::new(),
            mutation_fields_plan_order: Vec::new(),
        }
    }

    #[instrument(skip_all)]
    pub(super) fn plan(mut self) -> LogicalPlanningResult<OperationPlan> {
        tracing::trace!("Logical Planning");
        self.plan_all_fields()?;

        for modifier in &self.operation.response_modifiers {
            for &field_id in &self.operation[modifier.impacted_fields] {
                let selection_set_id = match modifier.rule {
                    ResponseModifierRule::AuthorizedParentEdge { .. } => {
                        self.operation[field_id].parent_selection_set_id()
                    }
                    ResponseModifierRule::AuthorizedEdgeChild { .. } => self.operation[field_id]
                        .selection_set_id()
                        .expect("Only an object/interface can be authorized here"),
                };

                self.selection_set_to_objects_must_be_tracked
                    .set(selection_set_id, true);
            }
        }

        let Self {
            operation,
            schema,
            mut logical_plans,
            field_to_logical_plan_id,
            field_to_solved_requirement,
            mutation_fields_plan_order,
            selection_set_to_objects_must_be_tracked,
            mut solved_requirements,
            mut dependents_builder,
            ..
        } = self;
        for plan in &mut logical_plans {
            plan.root_field_ids_ordered_by_parent_entity_id_then_position
                .sort_unstable_by_key(|id| {
                    let field = &operation[*id];
                    (
                        field.definition_id().map(|id| schema[id].parent_entity),
                        field.query_position(),
                    )
                });
        }

        solved_requirements.sort_unstable_by_key(|(id, _)| *id);
        tracing::trace!(
            "Solved requirements: {:?}",
            solved_requirements.iter().map(|(id, _)| id).collect::<Vec<_>>()
        );
        tracing::trace!("Field to solved requirements: {:?}", field_to_solved_requirement);

        dependents_builder.sort_unstable();
        let children = IdToMany::from_sorted_vec(dependents_builder.into_iter().dedup().collect());

        let mut plan = OperationPlan {
            solved_requirements,
            mutation_fields_plan_order,
            field_to_solved_requirement,
            selection_set_to_objects_must_be_tracked,
            field_to_logical_plan_id: field_to_logical_plan_id
                .into_iter()
                .enumerate()
                .map(|(i, maybe_logical_plan_id)| match maybe_logical_plan_id {
                    Some(logical_plan_id) => logical_plan_id,
                    None => {
                        let name = &operation.response_keys[operation.fields[i].response_key()];
                        unreachable!("No plan was associated with field:\n{name}");
                    }
                })
                .collect(),
            parent_count: {
                let mut parent_count = vec![0; logical_plans.len()];
                for (_, child) in children.as_ref() {
                    parent_count[usize::from(*child)] += 1;
                }
                parent_count
            },
            logical_plans,
            children,
            in_topological_order: Vec::new(),
        };

        plan.in_topological_order = sorted_plan_ids_by_topological_order(&plan);
        Ok(plan)
    }

    /// Step 1 of the planning, attributed all fields to a plan and satisfying their requirements.
    fn plan_all_fields(&mut self) -> LogicalPlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let introspection = self.schema.walker().introspection_metadata();

        let walker = self.walker();
        let (introspection_field_ids, field_ids): (Vec<_>, Vec<_>) = walker
            .selection_set()
            .as_ref()
            .field_ids_ordered_by_parent_entity_id_then_position
            .iter()
            .partition(|field_id| {
                if let Some(definition) = walker.walk(**field_id).definition() {
                    definition.is_resolvable_in(introspection.subgraph_id)
                } else {
                    true
                }
            });

        if !introspection_field_ids.is_empty() {
            self.push_plan(
                QueryPath::default(),
                introspection.resolver_id,
                self.operation.root_object_id.into(),
                &introspection_field_ids,
            )?;
        }

        if matches!(self.operation.ty, OperationType::Mutation) {
            self.plan_mutation(field_ids)?;
        } else {
            // Subscription are considered to be Queries for planning, they just happen to have
            // only one root field.
            self.plan_query(field_ids)?;
        }

        Ok(())
    }

    /// A query is simply treated as a plan boundary with no parent.
    fn plan_query(&mut self, field_ids: Vec<FieldId>) -> LogicalPlanningResult<()> {
        let id = self.operation.root_selection_set_id;
        SelectionSetLogicalPlanner::new(self, &QueryPath::default(), None).solve(id, None, Vec::new(), field_ids)
    }

    /// Mutation is a special case because root fields need to execute in order. So planning each
    /// field individually and setting up plan dependencies between them to ensures proper
    /// execution order.
    fn plan_mutation(&mut self, field_ids: Vec<FieldId>) -> LogicalPlanningResult<()> {
        let mut groups = field_ids
            .into_iter()
            .into_group_map_by(|id| self.operation[*id].response_key())
            .into_values()
            .collect::<Vec<_>>();
        // Ordering groups by their position in the query, ensuring proper ordering of plans.
        groups.sort_unstable_by_key(|field_ids| field_ids.iter().map(|id| self.operation[*id].query_position()).min());

        // FIXME: generates one plan per field, should be aggregated if consecutive fields can be
        // planned by a single resolver.
        for field_ids in groups {
            let field = &self.operation[field_ids[0]];
            let definition_id = field
                .definition_id()
                .expect("Introspection resolver should have taken metadata fields");

            let resolver = self
                .schema
                .walker()
                .walk(definition_id)
                .resolvers()
                .next()
                .ok_or_else(|| LogicalPlanningError::CouldNotPlanAnyField {
                    missing: vec![self.operation.response_keys[field.response_key()].to_string()],
                    query_path: vec![],
                })?;

            let plan_id = self.push_plan(
                QueryPath::default(),
                resolver.id(),
                self.operation.root_object_id.into(),
                &field_ids,
            )?;
            self.mutation_fields_plan_order.push(plan_id);
        }
        Ok(())
    }

    /// Obviously providable fields have no requirements and can be provided by the current
    /// resolver.
    fn grow_with_obviously_providable_subselections(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        field_ids: &[FieldId],
    ) -> LogicalPlanningResult<()> {
        self.attribute_fields(field_ids, logic.id());
        for id in field_ids {
            if let Some(selection_set_id) = self.operation[*id].selection_set_id() {
                let field = self.walker().walk(*id);
                let path = path.child(field.response_key());
                let definition_id = field.definition().expect("wouldn't have a subselection").id();
                let logic = logic.child(definition_id);
                self.plan_selection_set(&path, &logic, *id, definition_id, selection_set_id)?;
            }
        }

        Ok(())
    }

    /// Recursively traverse the operation to attribute all fields, planning a boundary if not all
    /// are providable by the current plan.
    ///
    /// The traversal order is important. We want the deepest selection sets to be planned first
    /// ensuring that when we plan a boundary (~selection set with missing fields) we have a
    /// complete picture of the providable fields. All of their fields and nested sub-selections
    /// will be already attributed to plan.
    fn plan_selection_set(
        &mut self,
        path: &QueryPath,
        logic: &PlanningLogic<'a>,
        parent_field_id: FieldId,
        parent_definition_id: FieldDefinitionId,
        selection_set_id: SelectionSetId,
    ) -> LogicalPlanningResult<()> {
        let walker = self.walker();
        let (obviously_plannable_field_ids, unplanned_field_ids): (Vec<_>, Vec<_>) = self.operation[selection_set_id]
            .field_ids_ordered_by_parent_entity_id_then_position
            .iter()
            .copied()
            .partition(|field_id| {
                if let Some(definition) = walker.walk(*field_id).definition() {
                    logic.is_providable(definition.id())
                        && !definition.has_required_fields(logic.resolver().subgraph_id())
                } else {
                    true
                }
            });

        self.grow_with_obviously_providable_subselections(path, logic, &obviously_plannable_field_ids)?;

        let parent_extra_requirements = self
            .schema
            .walk(parent_definition_id)
            .directives()
            .authorized()
            .fold(Default::default(), |acc, directive| {
                RequiredFieldSet::union_cow(acc, Cow::Borrowed(directive.node()))
            });
        if !unplanned_field_ids.is_empty() || !parent_extra_requirements.is_empty() {
            SelectionSetLogicalPlanner::new(self, path, Some(logic)).solve(
                selection_set_id,
                Some((parent_field_id, parent_extra_requirements)),
                obviously_plannable_field_ids,
                unplanned_field_ids,
            )?;
        }
        Ok(())
    }

    pub fn walker(&self) -> OperationWalker<'_, (), ()> {
        self.operation.walker_with(self.schema.walker())
    }

    pub fn push_plan(
        &mut self,
        query_path: QueryPath,
        resolver_id: ResolverDefinitionId,
        entity_id: EntityId,
        root_field_ids: &[FieldId],
    ) -> LogicalPlanningResult<LogicalPlanId> {
        let id = LogicalPlanId::from(self.logical_plans.len());
        tracing::trace!(
            "Creating {id} ({}): {}",
            self.schema.walk(resolver_id).name(),
            root_field_ids
                .iter()
                .format_with(", ", |id, f| f(&format_args!(
                    "{}",
                    self.walker().walk(*id).response_key_str()
                )))
                // with opentelemetry this string might be formatted more than once... Leading to a
                // panic with .format_with()
                .to_string()
        );
        self.logical_plans.push(LogicalPlan {
            resolver_id,
            entity_id,
            // Sorted at the end as may need to add extra fields.
            root_field_ids_ordered_by_parent_entity_id_then_position: root_field_ids.to_vec(),
        });
        let logic = PlanningLogic::new(id, self.schema.walk(resolver_id));
        self.grow_with_obviously_providable_subselections(&query_path, &logic, root_field_ids)?;
        Ok(id)
    }

    pub fn register_plan_child(&mut self, edge: ParentToChildEdge) {
        // Not a big deal if that happens, we would just ignore the edge. But would indicate a bug
        // somewhere.
        assert!(edge.parent != edge.child, "Self-dependency detected");
        self.dependents_builder.push((edge.parent, edge.child));
    }

    pub fn attribute_fields(&mut self, fields: &[FieldId], id: LogicalPlanId) {
        for field_id in fields {
            self[*field_id] = Some(id);
        }
    }
}

fn sorted_plan_ids_by_topological_order(plan: &OperationPlan) -> Vec<LogicalPlanId> {
    let mut parent_count = plan.parent_count.clone();
    let mut out = parent_count
        .iter()
        .enumerate()
        .filter_map(|(i, count)| {
            if *count == 0 {
                Some(LogicalPlanId::from(i))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut i = 0;
    while let Some(plan_id) = out.get(i) {
        for child in plan.children.find_all(*plan_id) {
            parent_count[usize::from(*child)] -= 1;
            if parent_count[usize::from(*child)] == 0 {
                out.push(*child);
            }
        }
        i += 1;
    }

    debug_assert_eq!(
        out.len(),
        plan.logical_plans.len(),
        "parent_count: {:?}\nchildren: {:?}\n -> {:?}",
        plan.parent_count,
        plan.children.as_ref(),
        out
    );
    out
}

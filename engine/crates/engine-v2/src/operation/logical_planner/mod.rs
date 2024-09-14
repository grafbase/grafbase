mod logic;
mod selection_set;

use std::borrow::Cow;

use engine_parser::types::OperationType;
use id_derives::IndexedFields;
use id_newtypes::{BitSet, IdToMany};
use itertools::Itertools;
use schema::{
    EntityDefinitionId, FieldDefinitionId, RequiredFieldId, RequiredFieldSetRecord, ResolverDefinitionId, Schema,
    TypeSystemDirective,
};

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
    /// A reference to the schema used for planning.
    schema: &'a Schema,

    /// A mutable reference to the current operation being planned.
    operation: &'a mut Operation,

    /// Maps each field ID to its corresponding logical plan ID, if one exists.
    #[indexed_by(FieldId)]
    field_to_logical_plan_id: Vec<Option<LogicalPlanId>>,

    /// Maps each field ID to its corresponding solved requirement ID, if one exists.
    field_to_solved_requirement: Vec<Option<RequiredFieldId>>,

    /// A collection of logical plans generated during the planning process.
    #[indexed_by(LogicalPlanId)]
    logical_plans: Vec<LogicalPlan>,

    /// A bitset indicating which selection sets must be tracked for object resolution.
    selection_set_to_objects_must_be_tracked: BitSet<SelectionSetId>,

    /// The order in which logical plans for mutation fields will be executed.
    mutation_fields_plan_order: Vec<LogicalPlanId>,

    /// A builder that holds dependencies between logical plans in terms of parent-child relationships.
    /// May have duplicates; parent may be equal to child (if we, as the supergraph, need the dependencies) (parent, child).
    dependents_builder: Vec<(LogicalPlanId, LogicalPlanId)>,

    /// A collection of solved requirements for selection sets, mapped to their IDs.
    solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

/// A struct representing a directed edge from a parent logical plan to a child logical plan.
///
/// This edge helps in maintaining the relationship between parent and child plans during the
/// logical planning process. It requires that any logical plan is distinct from its child plan
/// to avoid self-dependencies.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParentToChildEdge {
    /// The ID of the parent logical plan.
    pub parent: LogicalPlanId,
    /// The ID of the child logical plan.
    pub child: LogicalPlanId,
}

impl<'a> LogicalPlanner<'a> {
    /// Creates a new instance of `LogicalPlanner`.
    ///
    /// This function initializes the `LogicalPlanner` with the provided schema and operation.
    ///
    /// # Parameters
    ///
    /// - `schema`: A reference to the schema used for planning.
    /// - `operation`: A mutable reference to the current operation being planned.
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

    /// Plans the operation by generating a logical plan based on the current schema and operation.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult` containing the resulting `OperationPlan` if successful,
    /// or a `LogicalPlanningError` if planning fails.
    ///
    /// # Errors
    ///
    /// This function may return an error if any fields could not be planned or if there are issues with
    /// the response modifiers.
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
            plan.root_field_ids.sort_unstable_by_key(|id| {
                let field = &operation[*id];
                (
                    field.definition_id().map(|id| schema[id].parent_entity_id),
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

    /// Plans all fields for the current operation, attributing each field to a logical plan while
    /// satisfying their requirements. This function serves as the first step in the logical planning
    /// process and ensures that all fields are properly associated with their respective plans.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult` indicating success or failure. If successful,
    /// it will simply return `Ok(())`, otherwise, an error will be returned detailing which fields
    /// could not be planned or any issues encountered during the planning.
    fn plan_all_fields(&mut self) -> LogicalPlanningResult<()> {
        // The root plan is always introspection which also lets us handle operations like:
        // query { __typename }
        let introspection = &self.schema.subgraphs.introspection;

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

    /// Plans a query operation as a plan boundary, indicating that it has no parent in the logical planning structure.
    ///
    /// This function takes a vector of field IDs that represent the fields involved in the query.
    /// It sets up the necessary logical plan to execute these fields, ensuring the planning respects
    /// the structure of the provided schema and the current operation context.
    ///
    /// # Parameters
    ///
    /// - `field_ids`: A vector of field IDs that are part of the query operation being planned.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult<()>`, indicating whether the planning was
    /// successful or if an error occurred during the process.
    fn plan_query(&mut self, field_ids: Vec<FieldId>) -> LogicalPlanningResult<()> {
        let id = self.operation.root_selection_set_id;
        SelectionSetLogicalPlanner::new(self, &QueryPath::default(), None).solve(id, None, Vec::new(), field_ids)
    }

    /// Plans the mutation operation by establishing an execution order for the root fields.
    ///
    /// In a mutation, it is crucial for root fields to execute in a specified order. This function
    /// plans each field individually while setting up dependencies between them, ensuring that
    /// the execution order is correctly maintained throughout the mutation process.
    ///
    /// # Parameters
    ///
    /// - `field_ids`: A vector of field IDs representing the fields involved in the mutation.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult<()>`, indicating success or failure of the
    /// planning operation.
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

            let resolver = self.schema.walk(definition_id).resolvers().next().ok_or_else(|| {
                LogicalPlanningError::CouldNotPlanAnyField {
                    missing: vec![self.operation.response_keys[field.response_key()].to_string()],
                    query_path: vec![],
                }
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

    /// Grows the selection set by adding obviously providable sub-selections.
    ///
    /// This function identifies fields that have no requirements and can be resolved
    /// directly by the current resolver. For each of these fields, it attributes them to
    /// the logical planning context, and if the fields have nested selection sets, it
    /// plans those as well.
    ///
    /// # Parameters
    ///
    /// - `path`: The current path in the query being processed.
    /// - `logic`: The planning logic context used to determine field providability.
    /// - `field_ids`: A slice of field IDs that are being evaluated for sub-selections.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult<()>`, indicating success or failure
    /// of the planning operation.
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
                        && !definition.has_required_fields_for_subgraph(logic.resolver().subgraph_id())
                } else {
                    true
                }
            });

        self.grow_with_obviously_providable_subselections(path, logic, &obviously_plannable_field_ids)?;

        let parent_extra_requirements = self
            .schema
            .walk(parent_definition_id)
            .directives()
            .filter_map(|directive| match directive {
                TypeSystemDirective::Authorized(directive) => Some(directive),
                _ => None,
            })
            .fold(Default::default(), |acc, directive| {
                if let Some(node) = directive.node() {
                    RequiredFieldSetRecord::union_cow(acc, Cow::Borrowed(node.as_ref()))
                } else {
                    acc
                }
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

    /// Retrieves a walker for the current operation, allowing traversal and inspection of fields.
    pub fn walker(&self) -> OperationWalker<'_, ()> {
        self.operation.walker_with(self.schema)
    }

    /// Pushes a new logical plan into the planner with the specified parameters.
    ///
    /// This function creates a new logical plan associated with the given query path, resolver ID,
    /// entity ID, and root field IDs. It also attempts to grow the selection set by adding fields
    /// that can be providable by the resolver.
    ///
    /// # Parameters
    ///
    /// - `query_path`: The path in the query where this logical plan is being defined.
    /// - `resolver_id`: The ID of the resolver responsible for this logical plan.
    /// - `entity_id`: The ID of the entity related to this logical plan.
    /// - `root_field_ids`: A slice of field IDs that are the root fields for this logical plan.
    ///
    /// # Returns
    ///
    /// This function returns a `LogicalPlanningResult<LogicalPlanId>`, which contains the ID of the
    /// newly created logical plan if successful, or a `LogicalPlanningError` if there was a failure
    /// in planning.
    pub fn push_plan(
        &mut self,
        query_path: QueryPath,
        resolver_id: ResolverDefinitionId,
        entity_id: EntityDefinitionId,
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
            root_field_ids: root_field_ids.to_vec(),
        });
        let logic = PlanningLogic::new(id, self.schema, self.schema.walk(resolver_id));
        self.grow_with_obviously_providable_subselections(&query_path, &logic, root_field_ids)?;
        Ok(id)
    }

    /// Registers a child logical plan as a dependent of a parent logical plan.
    ///
    /// This function takes a directed edge representing the relationship between a parent
    /// logical plan and its child. If the parent and child are the same, it indicates a
    /// self-dependency, which is considered an error.
    ///
    /// # Parameters
    ///
    /// - `edge`: A `ParentToChildEdge` instance representing the parent-child relationship
    ///   between logical plans.
    ///
    /// # Panics
    ///
    /// This function will panic if the parent plan ID is the same as the child plan ID, indicating
    /// a self-dependency which is not allowed.
    pub fn register_plan_child(&mut self, edge: ParentToChildEdge) {
        // Not a big deal if that happens, we would just ignore the edge. But would indicate a bug
        // somewhere.
        assert!(edge.parent != edge.child, "Self-dependency detected");
        self.dependents_builder.push((edge.parent, edge.child));
    }

    /// Attributes the specified fields to a given logical plan ID.
    ///
    /// This function establishes a relationship between the provided field IDs and the specified
    /// logical plan ID, meaning that the given fields are mapped to the logical plan they belong to.
    ///
    /// # Parameters
    ///
    /// - `fields`: A slice of field IDs that need to be attributed to a logical plan.
    /// - `id`: The ID of the logical plan that the fields are associated with.
    pub fn attribute_fields(&mut self, fields: &[FieldId], id: LogicalPlanId) {
        for field_id in fields {
            self[*field_id] = Some(id);
        }
    }
}

/// Computes the logical plan IDs in topological order.
///
/// This function takes a reference to an `OperationPlan` and returns a vector of
/// `LogicalPlanId`s sorted in topological order. The sorting is based on the
/// parent-child relationship of the logical plans. Each logical plan can have
/// one or more child plans, and the topological order ensures that a plan appears
/// before any of its children in the sorted output.
///
/// # Parameters
///
/// - `plan`: A reference to the `OperationPlan` which contains the logical plans
///   and their relationships.
///
/// # Returns
///
/// A vector of `LogicalPlanId`s sorted in topological order.
///
/// # Panics
///
/// This function will panic if the topological sort cannot account for all logical plans,
/// indicating a cyclic dependency in the plan relationships.
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

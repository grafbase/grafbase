use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{Definition, RequiredFieldSet, Schema};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use crate::{
    operation::{
        FieldId, Operation, OperationWalker, PlanBoundaryId, PlanId, SelectionSetId, SelectionSetType, Variables,
    },
    plan::{
        flatten_selection_sets, AnyCollectedSelectionSet, AnyCollectedSelectionSetId, CollectedField, CollectedFieldId,
        CollectedSelectionSet, CollectedSelectionSetId, ConditionalField, ConditionalFieldId, ConditionalSelectionSet,
        ConditionalSelectionSetId, EntityId, ExecutionPlan, ExecutionPlanBoundaryId, ExecutionPlanId, FieldType,
        FlatField, OperationPlan, ParentToChildEdge, PlanInput, PlanOutput,
    },
    response::{ReadField, ReadSelectionSet},
    sources::PreparedExecutor,
};

use super::{PlanningError, PlanningResult};

pub(super) struct OperationPlanBuilder<'a> {
    schema: &'a Schema,
    variables: &'a Variables,
    operation_plan: OperationPlan,
    to_be_planned: Vec<ToBePlanned>,
    plan_parent_to_child_edges: HashSet<UnfinalizedParentToChildEdge>,
    plan_id_to_execution_plan_id: Vec<Option<ExecutionPlanId>>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct UnfinalizedParentToChildEdge {
    parent: PlanId,
    child: PlanId,
}

struct ToBePlanned {
    maybe_boundary_id: Option<ExecutionPlanBoundaryId>,
    plan_boundary_id: PlanBoundaryId,
    plan_id: PlanId,
    fields: Vec<FieldId>,
}

impl<'a> OperationPlanBuilder<'a> {
    pub(super) fn new(schema: &'a Schema, variables: &'a Variables, operation: Operation) -> Self {
        OperationPlanBuilder {
            schema,
            variables,
            to_be_planned: Vec::new(),
            plan_parent_to_child_edges: HashSet::new(),
            plan_id_to_execution_plan_id: vec![None; operation.plans.len()],
            operation_plan: OperationPlan {
                selection_set_to_collected: vec![None; operation.selection_sets.len()],
                execution_plans: Vec::new(),
                plan_parent_to_child_edges: Vec::new(),
                plan_dependencies_count: Vec::new(),
                plan_boundary_consummers_count: Vec::new(),
                conditional_selection_sets: Vec::new(),
                conditional_fields: Vec::new(),
                collected_selection_sets: Vec::new(),
                collected_fields: Vec::new(),
                operation,
            },
        }
    }

    pub(super) fn build(mut self) -> PlanningResult<OperationPlan> {
        self.generate_root_execution_plans()?;
        let mut operation_plan = self.operation_plan;
        operation_plan.plan_parent_to_child_edges = self
            .plan_parent_to_child_edges
            .into_iter()
            .map(|edge| {
                let parent = self.plan_id_to_execution_plan_id[usize::from(edge.parent)];
                let child = self.plan_id_to_execution_plan_id[usize::from(edge.child)];
                match (parent, child) {
                    (Some(parent), Some(child)) => Ok(ParentToChildEdge { parent, child }),
                    pc => Err(PlanningError::InternalError(format!(
                        "Unplanned depedency: {edge:?} -> {pc:?}"
                    ))),
                }
            })
            .collect::<Result<_, _>>()?;
        operation_plan.plan_parent_to_child_edges.sort_unstable();
        for ParentToChildEdge { child, .. } in &operation_plan.plan_parent_to_child_edges {
            operation_plan.plan_dependencies_count[usize::from(*child)] += 1;
        }
        tracing::trace!(
            "== Dependency Summary ==\nEdges:\n{}\nIncoming degree:\n{}",
            operation_plan
                .plan_parent_to_child_edges
                .iter()
                .format_with("\n", |edge, f| f(&format_args!("{} -> {}", edge.parent, edge.child))),
            operation_plan
                .plan_dependencies_count
                .iter()
                .enumerate()
                .format_with("\n", |(i, count), f| f(&format_args!(
                    "{} <- {}",
                    ExecutionPlanId::from(i),
                    count
                )))
        );

        Ok(operation_plan)
    }

    fn generate_root_execution_plans(&mut self) -> PlanningResult<()> {
        let walker = self.walker();
        let root_plans =
            walker
                .selection_set()
                .fields()
                .fold(HashMap::<PlanId, Vec<FieldId>>::default(), |mut acc, field| {
                    let plan_id = self.operation_plan.operation.field_to_plan_id[usize::from(field.id())]
                        .expect("Should be planned");
                    acc.entry(plan_id).or_default().push(field.id());
                    acc
                });
        if walker.is_mutation() {
            let mut maybe_previous_plan_id: Option<PlanId> = None;
            let mut plan_ids = root_plans
                .iter()
                .map(|(plan_id, fields)| (walker.walk(fields[0]).as_ref().query_position(), plan_id))
                .collect::<Vec<_>>();
            plan_ids.sort_unstable();
            for (_, &plan_id) in plan_ids {
                tracing::info!(
                    "Planning {} for {}",
                    plan_id,
                    self.walker().walk((&root_plans[&plan_id])[0]).response_key_str(),
                );
                if let Some(previous_plan_id) = maybe_previous_plan_id {
                    self.plan_parent_to_child_edges.insert(UnfinalizedParentToChildEdge {
                        parent: previous_plan_id,
                        child: plan_id,
                    });
                }
                maybe_previous_plan_id = Some(plan_id);
            }
        }

        self.to_be_planned = root_plans
            .into_iter()
            .map(|(plan_id, fields)| ToBePlanned {
                plan_boundary_id: PlanBoundaryId::from(0),
                maybe_boundary_id: None,
                plan_id,
                fields,
            })
            .collect();

        while let Some(ToBePlanned {
            maybe_boundary_id,
            plan_boundary_id,
            plan_id,
            fields,
        }) = self.to_be_planned.pop()
        {
            ExecutionPlanBuildContext::new(self, plan_boundary_id, plan_id).create_plan(maybe_boundary_id, fields)?;
        }

        Ok(())
    }

    fn walker(&self) -> OperationWalker<'_, (), ()> {
        // yes looks weird, will be improved
        self.operation_plan
            .operation
            .walker_with(self.schema.walker(), self.variables)
    }

    fn new_boundary(&mut self) -> ExecutionPlanBoundaryId {
        let id = ExecutionPlanBoundaryId::from(self.operation_plan.plan_boundary_consummers_count.len());
        self.operation_plan.plan_boundary_consummers_count.push(0);
        id
    }
}

pub(super) struct ExecutionPlanBuildContext<'parent, 'ctx> {
    builder: &'parent mut OperationPlanBuilder<'ctx>,
    plan_boundary_id: PlanBoundaryId,
    plan_id: PlanId,
    support_aliases: bool,
}

impl<'parent, 'ctx> std::ops::Deref for ExecutionPlanBuildContext<'parent, 'ctx> {
    type Target = OperationPlanBuilder<'ctx>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'parent, 'ctx> std::ops::DerefMut for ExecutionPlanBuildContext<'parent, 'ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.builder
    }
}

impl<'parent, 'ctx> ExecutionPlanBuildContext<'parent, 'ctx> {
    pub(super) fn new(
        builder: &'parent mut OperationPlanBuilder<'ctx>,
        plan_boundary_id: PlanBoundaryId,
        plan_id: PlanId,
    ) -> Self {
        let support_aliases = builder
            .schema
            .walk(builder.operation_plan.operation.plans[usize::from(plan_id)].resolver_id)
            .supports_aliases();
        ExecutionPlanBuildContext {
            builder,
            plan_boundary_id,
            plan_id,
            support_aliases,
        }
    }

    fn create_plan(
        &mut self,
        maybe_boundary_id: Option<ExecutionPlanBoundaryId>,
        fields: Vec<FieldId>,
    ) -> PlanningResult<()> {
        self.operation_plan.plan_dependencies_count.push(0);

        let input = if let Some(boundary_id) = maybe_boundary_id {
            self.operation_plan.plan_boundary_consummers_count[usize::from(boundary_id)] += 1;
            let selection_set = self.create_plan_input(&fields);
            Some(PlanInput {
                boundary_id,
                selection_set,
            })
        } else {
            None
        };

        // Currently a resolver is tied to only one entity (object/interface), so retrieving the
        // parent entity of any field is enough for this part.
        let selection_set_id = self.operation_plan.operation.parent_selection_set_id(fields[0]);
        let entity_id =
            EntityId::maybe_from(Definition::from(self.operation_plan.operation[selection_set_id].ty)).unwrap();

        let boundaries_start = self.operation_plan.plan_boundary_consummers_count.len();
        let collected_selection_set_id = self.collect_fields(entity_id.into(), fields, maybe_boundary_id)?;
        let boundaries_end = self.operation_plan.plan_boundary_consummers_count.len();
        let output = PlanOutput {
            entity_id,
            collected_selection_set_id,
            boundary_ids: IdRange::from(boundaries_start..boundaries_end),
        };

        let resolver_id = self.operation_plan.operation[self.plan_id].resolver_id;
        let resolver = self.schema.walker().walk(resolver_id).with_own_names();

        let plan_id = self.plan_id;
        let execution_plan = ExecutionPlan {
            plan_id,
            resolver_id,
            input,
            output,
            prepared_executor: PreparedExecutor::Unreachable,
        };
        self.operation_plan.execution_plans.push(execution_plan);
        let execution_plan_id = ExecutionPlanId::from(self.operation_plan.execution_plans.len() - 1);
        let prepared_executor = PreparedExecutor::prepare(
            resolver,
            self.operation_plan.ty,
            self.operation_plan
                .walker_with(self.schema, self.variables, execution_plan_id),
        )?;
        self.operation_plan.execution_plans[usize::from(execution_plan_id)].prepared_executor = prepared_executor;
        self.plan_id_to_execution_plan_id[usize::from(plan_id)] = Some(execution_plan_id);

        Ok(())
    }

    fn create_plan_input(&mut self, root_fields: &Vec<FieldId>) -> ReadSelectionSet {
        let resolver = self
            .schema
            .walk(self.operation_plan.operation[self.plan_id].resolver_id)
            .with_own_names();
        let mut requires = Cow::Borrowed(resolver.requires());
        for field_id in root_fields {
            if let Some(definition) = self.walker().walk(*field_id).definition() {
                let field_requires = definition.requires(resolver.subgraph_id());
                if !field_requires.is_empty() {
                    requires = Cow::Owned(requires.union(field_requires));
                }
            }
        }
        self.create_input_selection_set(&requires)
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_input_selection_set(&mut self, requires: &RequiredFieldSet) -> ReadSelectionSet {
        if requires.is_empty() {
            return ReadSelectionSet::default();
        }
        requires
            .iter()
            .map(|required_field| {
                let field_id = self
                    .operation_plan
                    .operation
                    .find_matching_field(self.plan_boundary_id, required_field.id)
                    .expect("Should be planned");
                let parent_plan_id = self.operation_plan.operation.field_to_plan_id[usize::from(field_id)]
                    .expect("field should be planned");
                let edge = UnfinalizedParentToChildEdge {
                    parent: parent_plan_id,
                    child: self.plan_id,
                };
                self.plan_parent_to_child_edges.insert(edge);
                let resolver = self
                    .schema
                    .walk(self.operation_plan.operation[self.plan_id].resolver_id)
                    .with_own_names();
                let f = ReadField {
                    edge: self.operation_plan.operation[field_id].response_edge(),
                    name: resolver
                        .walk(self.schema[required_field.id].definition_id)
                        .name()
                        .to_string(),
                    subselection: self.create_input_selection_set(&required_field.subselection),
                };
                tracing::info!("Plan {} depends on {} for {}", self.plan_id, parent_plan_id, f.name);
                f
            })
            .collect()
    }

    fn collect_selection_set(
        &mut self,
        selection_set_ids: Vec<SelectionSetId>,
        concrete_parent: bool,
    ) -> PlanningResult<AnyCollectedSelectionSet> {
        let selection_set = flatten_selection_sets(self.schema, &self.operation_plan, selection_set_ids.clone());

        let mut plan_fields = Vec::new();
        let mut children_plan: HashMap<PlanId, Vec<FieldId>> = HashMap::new();
        let mut maybe_plan_boundary_id = None;
        for field in selection_set.fields {
            if !self.operation_plan[field.id].is_read() {
                continue;
            }

            let field_plan_id =
                self.operation_plan.operation.field_to_plan_id[usize::from(field.id)].expect("Should be planned");
            if field_plan_id == self.plan_id {
                plan_fields.push(field);
            } else {
                if let Some(plan_boundary_id) = self
                    .operation_plan
                    .operation
                    .find_boundary_between(self.plan_id, field_plan_id)
                {
                    maybe_plan_boundary_id = Some(plan_boundary_id);
                }

                children_plan.entry(field_plan_id).or_default().push(field.id);
            }
        }
        let maybe_boundary_id = if children_plan.is_empty() {
            None
        } else {
            let maybe_boundary_id = Some(self.new_boundary());
            // Not all plans at this boundary necessarily have the current plan as a parent,
            // intermediate plans may also exist. But at least one of them will be a child.
            let plan_boundary_id = maybe_plan_boundary_id.expect("At least one plan must be a child");
            let to_be_planned = children_plan
                .into_iter()
                .map(|(plan_id, fields)| ToBePlanned {
                    maybe_boundary_id,
                    plan_boundary_id,
                    plan_id,
                    fields,
                })
                .collect::<Vec<_>>();
            self.to_be_planned.extend(to_be_planned);
            maybe_boundary_id
        };

        let is_union = selection_set_ids
            .iter()
            .any(|id| self.operation_plan[*id].ty.is_union());
        let unique_entity = !is_union && {
            let entities = plan_fields
                .iter()
                .flat_map(|field| field.entity_id)
                .chain(
                    selection_set_ids
                        .iter()
                        .flat_map(|id| self.operation_plan[*id].ty.as_entity_id()),
                )
                .collect::<HashSet<_>>();
            entities.len() == 1
        };

        // Trying to simplify the attributed selection to a concrete one.
        // - if the parent is not concrete, there might be other selection sets that need to be merged
        //   at runtime with this one.
        // - the only concrete selection set we support right now is one without any conditions.
        //   If a single condition is left, we can only work with None. A selection set like
        //   `animal { ... on Dog { name } }` would have a single condition, but we may still see
        //   cat objects. A ConcreteSelectionSet would require `name`.
        let id = if concrete_parent && unique_entity {
            self.collect_fields(
                selection_set.ty,
                plan_fields.into_iter().map(|field| field.id).collect(),
                maybe_boundary_id,
            )
            .map(AnyCollectedSelectionSetId::Collected)?
        } else {
            self.collected_conditional_fields(selection_set.ty, plan_fields, maybe_boundary_id)
                .map(AnyCollectedSelectionSetId::Conditional)?
        };

        // We keep track of which collected selection set matches which bound selection sets.
        // This allows us to know whether `__typename` is necessary in the generated subgraph query.
        for root_id in selection_set.root_selection_set_ids {
            self.operation_plan.selection_set_to_collected[usize::from(root_id)] = Some(id);
        }
        Ok(match id {
            AnyCollectedSelectionSetId::Collected(id) => AnyCollectedSelectionSet::Collected(id),
            AnyCollectedSelectionSetId::Conditional(id) => AnyCollectedSelectionSet::Conditional(id),
        })
    }

    fn collect_fields(
        &mut self,
        ty: SelectionSetType,
        fields: Vec<FieldId>,
        maybe_boundary_id: Option<ExecutionPlanBoundaryId>,
    ) -> PlanningResult<CollectedSelectionSetId> {
        let grouped_by_response_key = self
            .walker()
            .group_by_response_key_sorted_by_query_position(fields)
            .into_values();

        let mut fields = vec![];
        let mut typename_fields = vec![];
        for field_ids in grouped_by_response_key {
            let field_id: FieldId = field_ids[0];
            let field = self.operation_plan[field_id].clone();
            if let Some(definition_id) = field.definition_id() {
                let definition = self.schema.walk(definition_id);
                let expected_key = if self.support_aliases {
                    self.operation_plan.response_keys.ensure_safety(field.response_key())
                } else {
                    self.operation_plan.response_keys.get_or_intern(definition.name())
                };
                let ty = match definition.ty().inner().scalar_type() {
                    Some(scalar_type) => FieldType::Scalar(scalar_type),
                    None => {
                        let subselection_set_ids = field_ids
                            .into_iter()
                            .filter_map(|id| self.operation_plan[id].selection_set_id())
                            .collect();
                        FieldType::SelectionSet(self.collect_selection_set(subselection_set_ids, true)?)
                    }
                };
                fields.push(CollectedField {
                    expected_key,
                    edge: field.response_edge(),
                    id: field_id,
                    definition_id,
                    wrapping: definition.ty().wrapping(),
                    ty,
                });
            } else {
                typename_fields.push(field.response_edge());
            }
        }

        // Sorting by expected_key for deserialization
        let keys = &self.operation_plan.response_keys;
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        let field_ids = self.push_collecteded_fields(fields);
        Ok(self.push_collected_selection_set(CollectedSelectionSet {
            ty,
            maybe_boundary_id,
            field_ids,
            typename_fields,
        }))
    }

    fn collected_conditional_fields(
        &mut self,
        ty: SelectionSetType,
        flat_fields: Vec<FlatField>,
        maybe_boundary_id: Option<ExecutionPlanBoundaryId>,
    ) -> PlanningResult<ConditionalSelectionSetId> {
        let mut typename_fields = Vec::new();
        let mut conditional_fields = Vec::new();
        for flat_field in flat_fields {
            let field = self.operation_plan[flat_field.id].clone();
            if let Some(definition_id) = field.definition_id() {
                let definition = self.schema.walker().walk(definition_id);
                let expected_key = if self.support_aliases {
                    self.operation_plan.response_keys.ensure_safety(field.response_key())
                } else {
                    self.operation_plan.response_keys.get_or_intern(definition.name())
                };
                let ty = match definition.ty().inner().scalar_type() {
                    Some(data_type) => FieldType::Scalar(data_type),
                    None => {
                        let selection_set_id =
                            self.collect_selection_set(field.selection_set_id().into_iter().collect(), false)?;
                        let AnyCollectedSelectionSet::Conditional(selection_set_id) = selection_set_id else {
                            unreachable!("undetermined selection set cannot produce concrete selecitons");
                        };
                        FieldType::SelectionSet(selection_set_id)
                    }
                };
                conditional_fields.push(ConditionalField {
                    entity_id: definition.parent_entity(),
                    edge: field.response_edge(),
                    expected_key,
                    definition_id,
                    id: flat_field.id,
                    ty,
                });
            } else {
                let type_condition = flat_field.entity_id;
                typename_fields.push((type_condition, field.response_edge()));
            }
        }

        let field_ids = self.push_conditional_fields(conditional_fields);
        Ok(self.push_conditional_selection_set(ConditionalSelectionSet {
            ty,
            maybe_boundary_id,
            field_ids,
            typename_fields,
        }))
    }

    fn push_conditional_selection_set(&mut self, selection_set: ConditionalSelectionSet) -> ConditionalSelectionSetId {
        let id = ConditionalSelectionSetId::from(self.operation_plan.conditional_selection_sets.len());
        self.operation_plan.conditional_selection_sets.push(selection_set);
        id
    }

    fn push_conditional_fields(&mut self, fields: Vec<ConditionalField>) -> IdRange<ConditionalFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = ConditionalFieldId::from(self.operation_plan.conditional_fields.len());
        self.operation_plan.conditional_fields.extend(fields);
        IdRange {
            start,
            end: ConditionalFieldId::from(self.operation_plan.conditional_fields.len()),
        }
    }

    fn push_collected_selection_set(&mut self, selection_set: CollectedSelectionSet) -> CollectedSelectionSetId {
        let id = CollectedSelectionSetId::from(self.operation_plan.collected_selection_sets.len());
        self.operation_plan.collected_selection_sets.push(selection_set);
        id
    }

    fn push_collecteded_fields(&mut self, fields: Vec<CollectedField>) -> IdRange<CollectedFieldId> {
        // Can be empty when only __typename fields are present.
        if fields.is_empty() {
            return IdRange::empty();
        }
        let start = CollectedFieldId::from(self.operation_plan.collected_fields.len());
        self.operation_plan.collected_fields.extend(fields);
        IdRange {
            start,
            end: CollectedFieldId::from(self.operation_plan.collected_fields.len()),
        }
    }
}

use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{Definition, RequiredFieldSet};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use crate::{
    execution::ExecutionContext,
    operation::{
        Condition, ConditionResult, EntityLocation, FieldId, Operation, OperationWalker, PlanId, SelectionSetId,
        SelectionSetType, Variables,
    },
    plan::{
        flatten_selection_sets, AnyCollectedSelectionSet, AnyCollectedSelectionSetId, CollectedField, CollectedFieldId,
        CollectedSelectionSet, CollectedSelectionSetId, ConditionalField, ConditionalFieldId, ConditionalSelectionSet,
        ConditionalSelectionSetId, EntityId, ExecutionPlan, ExecutionPlanId, FieldError, FieldType, FlatField,
        OperationPlan, ParentToChildEdge, PlanInput, PlanOutput,
    },
    response::{GraphqlError, ReadField, ReadSelectionSet},
    sources::PreparedExecutor,
};

use super::{PlanningError, PlanningResult};

pub(crate) struct OperationPlanBuilder<'a> {
    ctx: ExecutionContext<'a>,
    variables: &'a Variables,
    operation_plan: OperationPlan,
    to_be_planned: Vec<ToBePlanned>,
    plan_parent_to_child_edges: HashSet<UnfinalizedParentToChildEdge>,
    plan_id_to_execution_plan_id: Vec<Option<ExecutionPlanId>>,
    condition_results: Vec<ConditionResult>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct UnfinalizedParentToChildEdge {
    parent: PlanId,
    child: PlanId,
}

struct ToBePlanned {
    entity_location: EntityLocation,
    plan_id: PlanId,
    root_fields: Vec<FieldId>,
}

impl<'a> OperationPlanBuilder<'a> {
    pub(crate) fn new(ctx: ExecutionContext<'a>, variables: &'a Variables, operation: Operation) -> Self {
        let entity_locations_count = operation
            .field_to_entity_location
            .iter()
            .filter_map(|el| el.map(usize::from))
            .max()
            .map(|n| n + 1)
            .unwrap_or_default();
        OperationPlanBuilder {
            ctx,
            variables,
            to_be_planned: Vec::new(),
            plan_parent_to_child_edges: HashSet::new(),
            plan_id_to_execution_plan_id: vec![None; operation.plans.len()],
            condition_results: Vec::new(),
            operation_plan: OperationPlan {
                selection_set_to_collected: vec![None; operation.selection_sets.len()],
                execution_plans: Vec::new(),
                plan_parent_to_child_edges: Vec::new(),
                plan_dependencies_count: Vec::new(),
                conditional_selection_sets: Vec::new(),
                conditional_fields: Vec::new(),
                collected_selection_sets: Vec::new(),
                collected_fields: Vec::new(),
                entities_consummers_count: vec![0; entity_locations_count],
                operation,
            },
        }
    }

    pub(crate) async fn build(mut self) -> PlanningResult<OperationPlan> {
        self.condition_results = self.evaluate_all_conditions().await?;
        self.finalize()
    }

    async fn evaluate_all_conditions(&self) -> PlanningResult<Vec<ConditionResult>> {
        let mut results = Vec::with_capacity(self.operation_plan.conditions.len());

        let is_anonymous = self.ctx.access_token().is_anonymous();
        let mut scopes = None;

        for condition in &self.operation_plan.conditions {
            let result = match condition {
                Condition::All(ids) => ids
                    .iter()
                    .map(|id| &results[usize::from(*id)])
                    .fold(ConditionResult::Include, |current, cond| current & cond),
                Condition::Authenticated => {
                    if is_anonymous {
                        ConditionResult::Errors(vec![GraphqlError::new("Unauthenticated")])
                    } else {
                        ConditionResult::Include
                    }
                }
                Condition::RequiresScopes(id) => {
                    let scopes = scopes.get_or_insert_with(|| {
                        self.ctx
                            .access_token()
                            .get_claim("scope")
                            .as_str()
                            .map(|scope| scope.split(' ').collect::<Vec<_>>())
                            .unwrap_or_default()
                    });

                    if self.ctx.schema.walk(*id).matches(scopes) {
                        ConditionResult::Include
                    } else {
                        ConditionResult::Errors(vec![GraphqlError::new("Not allowed: insufficient scopes")])
                    }
                }
                Condition::Authorized { directive_id, field_id } => {
                    let directive = &self.ctx.schema[*directive_id];
                    let arguments = self
                        .walker()
                        .walk(*field_id)
                        .arguments()
                        .with_selection_set(&directive.arguments);
                    let input = crate::execution::hooks::authorized::Input { arguments };
                    if let Some(err) = self.ctx.hooks().authorized(input).await {
                        ConditionResult::Errors(vec![err])
                    } else {
                        ConditionResult::Include
                    }
                }
            };
            results.push(result);
        }

        Ok(results)
    }

    fn finalize(mut self) -> PlanningResult<OperationPlan> {
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
            .map(|(plan_id, root_fields)| ToBePlanned {
                entity_location: self
                    .walker()
                    .walk(root_fields[0])
                    .entity_location()
                    .expect("Should exist"),
                plan_id,
                root_fields,
            })
            .collect();

        while let Some(to_be_planned) = self.to_be_planned.pop() {
            self.generate_plan(to_be_planned)?;
        }

        Ok(())
    }

    fn generate_plan(
        &mut self,
        ToBePlanned {
            entity_location,
            plan_id,
            root_fields,
        }: ToBePlanned,
    ) -> PlanningResult<()> {
        let execution_plan = ExecutionPlanBuilder::new(self, plan_id).build(entity_location, root_fields)?;
        let resolver = self
            .ctx
            .schema
            .walker()
            .walk(execution_plan.resolver_id)
            .with_own_names();

        self.operation_plan.execution_plans.push(execution_plan);
        let execution_plan_id = ExecutionPlanId::from(self.operation_plan.execution_plans.len() - 1);
        let prepared_executor = PreparedExecutor::prepare(
            resolver,
            self.operation_plan.ty,
            self.operation_plan
                .walker_with(&self.ctx.schema, self.variables, execution_plan_id),
        )?;
        self.operation_plan.execution_plans[usize::from(execution_plan_id)].prepared_executor = prepared_executor;
        self.plan_id_to_execution_plan_id[usize::from(plan_id)] = Some(execution_plan_id);

        Ok(())
    }

    fn walker(&self) -> OperationWalker<'_, (), ()> {
        // yes looks weird, will be improved
        self.operation_plan
            .operation
            .walker_with(self.ctx.schema.walker(), self.variables)
    }
}

pub(super) struct ExecutionPlanBuilder<'parent, 'ctx> {
    builder: &'parent mut OperationPlanBuilder<'ctx>,
    plan_id: PlanId,
    support_aliases: bool,
    tracked_entity_locations: Vec<EntityLocation>,
}

impl<'parent, 'ctx> std::ops::Deref for ExecutionPlanBuilder<'parent, 'ctx> {
    type Target = OperationPlanBuilder<'ctx>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl<'parent, 'ctx> std::ops::DerefMut for ExecutionPlanBuilder<'parent, 'ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.builder
    }
}

impl<'parent, 'ctx> ExecutionPlanBuilder<'parent, 'ctx> {
    pub(super) fn new(builder: &'parent mut OperationPlanBuilder<'ctx>, plan_id: PlanId) -> Self {
        let support_aliases = builder
            .ctx
            .schema
            .walk(builder.operation_plan.operation.plans[usize::from(plan_id)].resolver_id)
            .supports_aliases();
        ExecutionPlanBuilder {
            builder,
            plan_id,
            support_aliases,
            tracked_entity_locations: Vec::new(),
        }
    }

    fn build(mut self, entity_location: EntityLocation, root_fields: Vec<FieldId>) -> PlanningResult<ExecutionPlan> {
        self.operation_plan.plan_dependencies_count.push(0);
        self.operation_plan.entities_consummers_count[usize::from(entity_location)] += 1;

        let input = PlanInput {
            entity_location,
            selection_set: self.create_plan_input(entity_location, &root_fields),
        };

        // Currently a resolver is tied to only one entity (object/interface), so retrieving the
        // parent entity of any field is enough for this part.
        let selection_set_id = self.operation_plan.operation.parent_selection_set_id(root_fields[0]);
        let entity_id =
            EntityId::maybe_from(Definition::from(self.operation_plan.operation[selection_set_id].ty)).unwrap();

        let collected_selection_set_id = self.collect_fields(entity_id.into(), None, root_fields)?;
        let Self {
            builder,
            plan_id,
            tracked_entity_locations,
            ..
        } = self;

        let output = PlanOutput {
            entity_id,
            collected_selection_set_id,
            tracked_entity_locations,
        };
        let resolver_id = builder.operation_plan.operation[self.plan_id].resolver_id;

        Ok(ExecutionPlan {
            plan_id,
            resolver_id,
            input,
            output,
            prepared_executor: PreparedExecutor::Unreachable,
        })
    }

    fn create_plan_input(&mut self, entity_location: EntityLocation, root_fields: &Vec<FieldId>) -> ReadSelectionSet {
        let resolver = self
            .ctx
            .engine
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
        self.create_input_selection_set(entity_location, &requires)
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_input_selection_set(
        &mut self,
        entity_location: EntityLocation,
        requires: &RequiredFieldSet,
    ) -> ReadSelectionSet {
        if requires.is_empty() {
            return ReadSelectionSet::default();
        }
        requires
            .iter()
            .map(|required_field| {
                let field_id = self
                    .operation_plan
                    .operation
                    .find_matching_field(entity_location, required_field.id)
                    .expect("Should be planned");
                let parent_plan_id = self.operation_plan.operation.field_to_plan_id[usize::from(field_id)]
                    .expect("field should be planned");
                let edge = UnfinalizedParentToChildEdge {
                    parent: parent_plan_id,
                    child: self.plan_id,
                };
                self.plan_parent_to_child_edges.insert(edge);
                let resolver = self
                    .ctx
                    .schema
                    .walk(self.operation_plan.operation[self.plan_id].resolver_id)
                    .with_own_names();
                ReadField {
                    edge: self.operation_plan.operation[field_id].response_edge(),
                    name: resolver
                        .walk(self.ctx.schema[required_field.id].definition_id)
                        .name()
                        .to_string(),
                    subselection: self.create_input_selection_set(entity_location, &required_field.subselection),
                }
            })
            .collect()
    }

    fn collect_selection_set(
        &mut self,
        selection_set_ids: Vec<SelectionSetId>,
        concrete_parent: bool,
    ) -> PlanningResult<AnyCollectedSelectionSet> {
        let selection_set = flatten_selection_sets(&self.ctx.schema, &self.operation_plan, selection_set_ids.clone());

        let mut plan_fields = Vec::new();
        let mut children_plan: HashMap<PlanId, Vec<FieldId>> = HashMap::new();
        for field in selection_set.fields {
            if !self.operation_plan[field.id].is_read() {
                continue;
            }

            let field_plan_id =
                self.operation_plan.operation.field_to_plan_id[usize::from(field.id)].expect("Should be planned");
            if field_plan_id == self.plan_id {
                plan_fields.push(field);
            } else {
                children_plan.entry(field_plan_id).or_default().push(field.id);
            }
        }
        let maybe_tracked_entity_location = if children_plan.is_empty() {
            None
        } else {
            let entity_location = {
                let field_id = children_plan.values().flatten().next().unwrap();
                self.walker().walk(*field_id).entity_location().expect("Should exist")
            };
            self.tracked_entity_locations.push(entity_location);
            let to_be_planned = children_plan
                .into_iter()
                .map(|(plan_id, fields)| ToBePlanned {
                    entity_location,
                    plan_id,
                    root_fields: fields,
                })
                .collect::<Vec<_>>();
            self.to_be_planned.extend(to_be_planned);
            Some(entity_location)
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
                maybe_tracked_entity_location,
                plan_fields.into_iter().map(|field| field.id).collect(),
            )
            .map(AnyCollectedSelectionSetId::Collected)?
        } else {
            self.collected_conditional_fields(selection_set.ty, maybe_tracked_entity_location, plan_fields)
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
        maybe_tracked_entity_location: Option<EntityLocation>,
        fields: Vec<FieldId>,
    ) -> PlanningResult<CollectedSelectionSetId> {
        let grouped_by_response_key = self
            .walker()
            .group_by_response_key_sorted_by_query_position(fields)
            .into_values();

        let mut fields = Vec::new();
        let mut typename_fields = Vec::new();
        let mut field_errors = Vec::new();
        for field_ids in grouped_by_response_key {
            let field_id: FieldId = field_ids[0];
            let field = self.operation_plan[field_id].clone();
            let Some(definition_id) = field.definition_id() else {
                typename_fields.push(field.response_edge());
                continue;
            };
            let definition = self.ctx.engine.schema.walk(definition_id);

            tracing::trace!("Collecting field {} with condition: {:#?}", definition.name(), {
                field.condition().map(|id| &self.condition_results[usize::from(id)])
            });
            match field.condition().map(|id| &self.condition_results[usize::from(id)]) {
                Some(ConditionResult::Errors(errors)) => {
                    field_errors.push(FieldError {
                        edge: field.response_edge(),
                        errors: errors.clone(),
                        is_required: definition.ty().wrapping().is_required(),
                    });
                }
                Some(ConditionResult::Include) | None => {}
            }

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
        }

        // Sorting by expected_key for deserialization
        let keys = &self.operation_plan.response_keys;
        fields.sort_unstable_by(|a, b| keys[a.expected_key].cmp(&keys[b.expected_key]));
        let field_ids = self.push_collecteded_fields(fields);
        Ok(self.push_collected_selection_set(CollectedSelectionSet {
            ty,
            maybe_tracked_entity_location,
            field_ids,
            typename_fields,
            field_errors,
        }))
    }

    fn collected_conditional_fields(
        &mut self,
        ty: SelectionSetType,
        maybe_tracked_entity_location: Option<EntityLocation>,
        flat_fields: Vec<FlatField>,
    ) -> PlanningResult<ConditionalSelectionSetId> {
        let mut typename_fields = Vec::new();
        let mut conditional_fields = Vec::new();
        let mut field_errors = Vec::new();
        for flat_field in flat_fields {
            let field = self.operation_plan[flat_field.id].clone();
            let Some(definition_id) = field.definition_id() else {
                let type_condition = flat_field.entity_id;
                typename_fields.push((type_condition, field.response_edge()));
                continue;
            };
            let definition = self.ctx.engine.schema.walk(definition_id);

            match field.condition().map(|id| &self.condition_results[usize::from(id)]) {
                Some(ConditionResult::Errors(errors)) => {
                    let field_error = FieldError {
                        edge: field.response_edge(),
                        errors: errors.clone(),
                        is_required: definition.ty().wrapping().is_required(),
                    };
                    field_errors.push((definition.parent_entity(), field_error));
                }
                Some(ConditionResult::Include) | None => {}
            }

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
        }

        let field_ids = self.push_conditional_fields(conditional_fields);
        Ok(self.push_conditional_selection_set(ConditionalSelectionSet {
            ty,
            maybe_tracked_entity_location,
            field_ids,
            typename_fields,
            field_errors,
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

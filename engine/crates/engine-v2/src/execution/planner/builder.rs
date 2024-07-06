use std::{borrow::Cow, collections::HashMap};

use id_newtypes::IdRange;
use schema::RequiredFieldSet;

use crate::{
    execution::{ExecutionPlan, PlanInput, PlanOutput},
    operation::{FieldId, LogicalPlanId, SelectionSetId, SelectionSetType},
    response::{ReadField, ReadSelectionSet, ResponseObjectSetId},
    Runtime,
};

use super::{ExecutionPlanner, PlanningResult, UnfinalizedParentToChildEdge};

pub(super) struct ExecutionPlanBuilder<'ctx, 'op, 'planner, R: Runtime> {
    pub(super) planner: &'planner mut ExecutionPlanner<'ctx, 'op, R>,
    pub(super) logical_plan_id: LogicalPlanId,
    pub(super) input_id: ResponseObjectSetId,
    pub(super) tracked_output_ids: Vec<ResponseObjectSetId>,
    pub(super) requires_typename_for: Vec<SelectionSetId>,
}

impl<'ctx, 'op, 'planner, R: Runtime> std::ops::Deref for ExecutionPlanBuilder<'ctx, 'op, 'planner, R> {
    type Target = ExecutionPlanner<'ctx, 'op, R>;
    fn deref(&self) -> &Self::Target {
        self.planner
    }
}

impl<'ctx, 'op, 'planner, R: Runtime> std::ops::DerefMut for ExecutionPlanBuilder<'ctx, 'op, 'planner, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.planner
    }
}

impl<'ctx, 'op, 'planner, R: Runtime> ExecutionPlanBuilder<'ctx, 'op, 'planner, R>
where
    'ctx: 'op,
{
    pub(super) fn new(
        planner: &'planner mut ExecutionPlanner<'ctx, 'op, R>,
        input_id: ResponseObjectSetId,
        logical_plan_id: LogicalPlanId,
    ) -> Self {
        ExecutionPlanBuilder {
            planner,
            input_id,
            logical_plan_id,
            tracked_output_ids: Vec::new(),
            requires_typename_for: Vec::new(),
        }
    }

    pub(super) fn build(
        mut self,
        selection_set_ty: SelectionSetType,
        mut root_field_ids: Vec<FieldId>,
    ) -> PlanningResult<ExecutionPlan> {
        let n = usize::from(self.input_id);
        self.plans.response_object_set_consummers_count[n] += 1;

        let input = PlanInput {
            id: self.input_id,
            entity_id: root_field_ids
                .iter()
                .find_map(|id| self.walker().walk(*id).definition().map(|def| def.parent_entity().id()))
                .or_else(|| selection_set_ty.as_entity_id())
                .expect("Should have at least one non __typename field at the root for nested plans, root ones execute on root objects"),
            requires: self.create_read_selection_set(&root_field_ids),
            dependencies_count: 0,
        };

        let shape_id = self.create_root_shape_for(&input, &root_field_ids)?;

        let schema = self.schema();
        root_field_ids.sort_unstable_by_key(|id| {
            let field = &self.operation[*id];
            (
                field.definition_id().map(|id| schema[id].parent_entity),
                field.query_position(),
            )
        });

        self.requires_typename_for.sort_unstable();
        let output = PlanOutput {
            root_field_ids_ordered_by_parent_entity_id_then_position: root_field_ids,
            shape_id,
            tracked_output_ids: IdRange::from_slice(&self.tracked_output_ids).expect("Contiguous ids"),
            dependent: Vec::new(),
            requires_typename_for: self.requires_typename_for,
        };

        Ok(ExecutionPlan {
            logical_plan_id: self.logical_plan_id,
            input,
            output,
        })
    }

    fn create_read_selection_set(&mut self, field_ids: &Vec<FieldId>) -> ReadSelectionSet {
        let resolver = self
            .ctx
            .engine
            .schema
            .walk(self.operation[self.logical_plan_id].resolver_id)
            .with_own_names();
        let mut field_ids_by_selection_set_id = HashMap::<_, Vec<_>>::new();
        for field_id in field_ids {
            field_ids_by_selection_set_id
                .entry(self.operation[*field_id].parent_selection_set_id())
                .or_default()
                .push(field_id);
        }

        let mut field_ids_by_selection_set_id = field_ids_by_selection_set_id.into_iter();

        let mut read_selection_set = {
            let (selection_set_id, field_ids) = field_ids_by_selection_set_id
                .next()
                .expect("At least one field is planned");
            let mut requires = Cow::Borrowed(resolver.requires());
            for field_id in field_ids {
                if let Some(definition) = self.walker().walk(*field_id).definition() {
                    let field_requires = definition.requires(resolver.subgraph_id());
                    if !field_requires.is_empty() {
                        requires = Cow::Owned(requires.union(field_requires));
                    }
                }
            }
            self.create_read_selection_set_for_requirements(selection_set_id, &requires)
        };

        for (selection_set_id, field_ids) in field_ids_by_selection_set_id {
            let mut requires = RequiredFieldSet::default();
            for field_id in field_ids {
                if let Some(definition) = self.walker().walk(*field_id).definition() {
                    let field_requires = definition.requires(resolver.subgraph_id());
                    if !field_requires.is_empty() {
                        requires = requires.union(field_requires);
                    }
                }
            }
            read_selection_set =
                read_selection_set.union(self.create_read_selection_set_for_requirements(selection_set_id, &requires));
        }

        read_selection_set
    }

    /// Create the input selection set of a Plan given its resolver and requirements.
    /// We iterate over the requirements and find the matching fields inside the boundary fields,
    /// which contains all providable & extra fields. During the iteration we track all the dependency
    /// plans.
    fn create_read_selection_set_for_requirements(
        &mut self,
        id: SelectionSetId,
        requires: &RequiredFieldSet,
    ) -> ReadSelectionSet {
        if requires.is_empty() {
            return ReadSelectionSet::default();
        }
        requires
            .iter()
            .map(|required_field| {
                let solved_requirements = &self.operation.solved_requirements_for(id).expect("Should be planned");
                let solved = solved_requirements
                    .iter()
                    .find(|req| req.id == required_field.id)
                    .expect("Solver did its job");
                let field_id = solved.field_id;
                let parent_plan_id = self.operation.plan_id_for(field_id);
                let edge = UnfinalizedParentToChildEdge {
                    parent: parent_plan_id,
                    child: self.logical_plan_id,
                };
                self.plan_parent_to_child_edges.insert(edge);
                let resolver = self
                    .ctx
                    .schema
                    .walk(self.operation[self.logical_plan_id].resolver_id)
                    .with_own_names();
                ReadField {
                    edge: self.operation[field_id].response_edge(),
                    name: resolver
                        .walk(self.ctx.schema[required_field.id].definition_id)
                        .name()
                        .to_string(),
                    subselection: if !required_field.subselection.is_empty() {
                        self.create_read_selection_set_for_requirements(
                            self.operation[field_id]
                                .selection_set_id()
                                .expect("Could not have requirements"),
                            &required_field.subselection,
                        )
                    } else {
                        ReadSelectionSet::default()
                    },
                }
            })
            .collect()
    }
}

mod partition;
mod plan;

use std::collections::HashMap;

use id_newtypes::BitSet;
use plan::LogicalPlanResponseBlueprintBuilder;
use schema::Schema;

use crate::{
    operation::{LogicalPlanResponseBlueprint, Operation, Variables},
    response::{FieldShape, FieldShapeId, ResponseObjectSetId},
    utils::BufferPool,
};

use super::{FieldId, LogicalPlanId, OperationPlan, OperationWalker, ResponseBlueprint, SelectionSetType};

pub(super) struct ResponseBlueprintBuilder<'schema, 'op> {
    schema: &'schema Schema,
    variables: &'op Variables,
    operation: &'op Operation,
    plan: &'op OperationPlan,
    to_build_stack: Vec<ToBuild>,
    field_shapes_buffer_pool: BufferPool<(FieldShape, Vec<FieldId>)>,
    field_id_to_field_shape_ids_builder: Vec<(FieldId, FieldShapeId)>,
    logical_plan_to_blueprint_builder: Vec<(LogicalPlanId, LogicalPlanResponseBlueprint)>,
    selection_set_to_response_object_set: Vec<Option<ResponseObjectSetId>>,
    blueprint: ResponseBlueprint,
}

struct ToBuild {
    logical_plan_id: LogicalPlanId,
    input_id: ResponseObjectSetId,
    root_field_ids: Vec<FieldId>,
}

impl<'schema, 'op> ResponseBlueprintBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn new(
        schema: &'schema Schema,
        variables: &'op Variables,
        operation: &'op Operation,
        plan: &'op OperationPlan,
    ) -> Self {
        ResponseBlueprintBuilder {
            schema,
            variables,
            operation,
            plan,
            to_build_stack: Vec::new(),
            blueprint: ResponseBlueprint {
                shapes: Default::default(),
                field_to_shape_ids: Default::default(),
                logical_plan_to_blueprint: Default::default(),
                selection_set_to_requires_typename: BitSet::init_with(false, operation.selection_sets.len()),
                response_modifier_impacted_field_to_response_object_set: Vec::new(),
                response_object_sets_to_type: Vec::new(),
            },
            field_shapes_buffer_pool: Default::default(),
            field_id_to_field_shape_ids_builder: Default::default(),
            logical_plan_to_blueprint_builder: Default::default(),
            selection_set_to_response_object_set: vec![None; operation.selection_sets.len()],
        }
    }

    pub(super) fn build(mut self) -> ResponseBlueprint {
        self.traverse_operation_and_build_blueprint();
        let Self {
            operation,
            mut blueprint,
            field_id_to_field_shape_ids_builder,
            selection_set_to_response_object_set,
            mut logical_plan_to_blueprint_builder,
            ..
        } = self;
        for &field_id in &operation.response_modifier_impacted_fields {
            let set_id = selection_set_to_response_object_set
                [usize::from(operation[field_id].parent_selection_set_id())]
            .expect("Not ResponseObjectSet defined for selection set");
            blueprint
                .response_modifier_impacted_field_to_response_object_set
                .push(set_id);
        }
        blueprint.field_to_shape_ids = field_id_to_field_shape_ids_builder.into();
        logical_plan_to_blueprint_builder.sort_unstable_by_key(|(id, _)| *id);
        blueprint.logical_plan_to_blueprint = logical_plan_to_blueprint_builder
            .into_iter()
            .map(|(_, bp)| bp)
            .collect();
        blueprint
    }

    fn traverse_operation_and_build_blueprint(&mut self) {
        let walker = self.walker();
        let root_plans = walker.selection_set().fields().fold(
            HashMap::<LogicalPlanId, Vec<FieldId>>::default(),
            |mut acc, field| {
                let plan_id = self.plan.field_to_logical_plan_id[usize::from(field.id())];
                acc.entry(plan_id).or_default().push(field.id());
                acc
            },
        );

        let input_id = self.next_response_object_set_id(SelectionSetType::Object(self.operation.root_object_id));
        self.to_build_stack = root_plans
            .into_iter()
            .map(|(logical_plan_id, root_field_ids)| ToBuild {
                input_id,
                logical_plan_id,
                root_field_ids,
            })
            .collect();

        while let Some(to_build) = self.to_build_stack.pop() {
            self.build_logical_plan_response_blueprint(to_build);
        }
    }

    fn build_logical_plan_response_blueprint(&mut self, to_build: ToBuild) {
        tracing::trace!("Generating blueprint for {}", to_build.logical_plan_id);
        let blueprint = LogicalPlanResponseBlueprintBuilder::build(self, &to_build);
        self.logical_plan_to_blueprint_builder
            .push((to_build.logical_plan_id, blueprint))
    }

    pub fn walker(&self) -> OperationWalker<'op, (), ()> {
        self.operation.walker_with(self.schema.walker(), self.variables)
    }

    fn next_response_object_set_id(&mut self, ty: SelectionSetType) -> ResponseObjectSetId {
        self.blueprint.response_object_sets_to_type.push(ty);
        (self.blueprint.response_object_sets_to_type.len() - 1).into()
    }
}

mod bind;
mod blueprint;
mod build;
pub mod ids;
mod input_value;
mod location;
mod logical_planner;
mod metrics;
mod modifier;
mod parse;
mod path;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use crate::response::{ConcreteObjectShapeId, FieldShapeId, ResponseKeys, ResponseObjectSetId, Shapes};
pub(crate) use engine_parser::types::OperationType;
use grafbase_telemetry::metrics::OperationMetricsAttributes;
use id_newtypes::{BitSet, IdRange, IdToMany};
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use modifier::*;
pub(crate) use path::QueryPath;
use schema::{EntityId, ObjectId, RequiredFieldId, ResolverId, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedOperation {
    pub operation: Operation,
    pub metrics_attributes: OperationMetricsAttributes,
    pub plan: OperationPlan,
    pub response_blueprint: ResponseBlueprint,
}

impl std::ops::Deref for PreparedOperation {
    type Target = Operation;
    fn deref(&self) -> &Self::Target {
        &self.operation
    }
}

impl<I> std::ops::Index<I> for PreparedOperation
where
    Operation: std::ops::Index<I>,
{
    type Output = <Operation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.operation[index]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    pub root_selection_set_id: SelectionSetId,
    // sorted
    pub root_query_modifier_ids: Vec<QueryModifierId>,
    pub response_keys: ResponseKeys,
    pub selection_sets: Vec<SelectionSet>,
    pub fields: Vec<Field>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub field_arguments: Vec<FieldArgument>,
    pub query_input_values: QueryInputValues,
    // deduplicated by rule
    pub query_modifiers: Vec<QueryModifier>,
    pub query_modifier_impacted_fields: Vec<FieldId>,
    // deduplicated by rule
    pub response_modifiers: Vec<ResponseModifier>,
    pub response_modifier_impacted_fields: Vec<FieldId>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct OperationPlan {
    pub field_to_logical_plan_id: Vec<LogicalPlanId>,
    pub field_to_solved_requirement: Vec<Option<RequiredFieldId>>,
    pub selection_set_to_objects_must_be_tracked: BitSet<SelectionSetId>,
    pub logical_plans: Vec<LogicalPlan>,
    pub mutation_fields_plan_order: Vec<LogicalPlanId>,
    pub children: IdToMany<LogicalPlanId, LogicalPlanId>,
    // LogicalPlanId -> u16
    pub parent_count: Vec<u16>,
    // All dependencies of a plan are placed before it.
    pub in_topological_order: Vec<LogicalPlanId>,
    // Sorted
    pub solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct LogicalPlan {
    pub resolver_id: ResolverId,
    pub entity_id: EntityId,
    pub root_field_ids_ordered_by_parent_entity_id_then_position: Vec<FieldId>,
}

pub(crate) type SolvedRequiredFieldSet = Vec<SolvedRequiredField>;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SolvedRequiredField {
    pub id: RequiredFieldId,
    pub field_id: FieldId,
    pub subselection: SolvedRequiredFieldSet,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseBlueprint {
    pub shapes: Shapes,
    pub field_to_shape_ids: IdToMany<FieldId, FieldShapeId>,
    pub response_modifier_impacted_field_to_response_object_set: Vec<ResponseObjectSetId>,
    pub logical_plan_to_blueprint: Vec<LogicalPlanResponseBlueprint>,
    pub selection_set_to_requires_typename: BitSet<SelectionSetId>,
    pub response_object_sets_to_type: Vec<SelectionSetType>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct LogicalPlanResponseBlueprint {
    pub input_id: ResponseObjectSetId,
    pub output_ids: IdRange<ResponseObjectSetId>,
    pub concrete_shape_id: ConcreteObjectShapeId,
}

impl<I> std::ops::Index<I> for ResponseBlueprint
where
    Shapes: std::ops::Index<I>,
{
    type Output = <Shapes as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.shapes[index]
    }
}

impl<I> std::ops::IndexMut<I> for ResponseBlueprint
where
    Shapes: std::ops::IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.shapes[index]
    }
}

impl PreparedOperation {
    pub fn ty(&self) -> OperationType {
        self.operation.ty
    }

    pub fn plan_id_for(&self, id: FieldId) -> LogicalPlanId {
        self.plan.field_to_logical_plan_id[usize::from(id)]
    }

    pub fn solved_requirements_for(&self, id: SelectionSetId) -> Option<&SolvedRequiredFieldSet> {
        self.plan
            .solved_requirements
            .binary_search_by(|probe| probe.0.cmp(&id))
            .map(|ix| &self.plan.solved_requirements[ix].1)
            .ok()
    }
}

impl Operation {
    pub fn walker_with<'op, 'schema, SI>(
        &'op self,
        schema_walker: SchemaWalker<'schema, SI>,
    ) -> OperationWalker<'op, (), SI>
    where
        'schema: 'op,
    {
        OperationWalker {
            operation: self,
            schema_walker,
            item: (),
        }
    }
}

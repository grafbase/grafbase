mod bind;
pub(crate) mod blueprint;
mod cache_scopes;
pub mod ids;
mod input_value;
mod location;
mod logical_planner;
mod metrics;
mod modifier;
mod parse;
mod path;
mod prepare;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use crate::response::{ConcreteObjectShapeId, FieldShapeId, ResponseKeys, ResponseObjectSetId, Shapes};
pub(crate) use bind::bind_operation;
pub(crate) use cache_scopes::*;
pub(crate) use engine_parser::types::OperationType;
use grafbase_telemetry::graphql::GraphqlOperationAttributes;
use id_derives::IndexedFields;
use id_newtypes::{BitSet, IdRange, IdToMany};
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use modifier::*;
pub(crate) use parse::{parse_operation, ParsedOperation};
pub(crate) use path::QueryPath;
use schema::{EntityDefinitionId, ObjectDefinitionId, RequiredFieldId, ResolverDefinitionId, Schema};
pub(crate) use selection_set::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PreparedOperation {
    pub operation: Operation,
    pub attributes: GraphqlOperationAttributes,
    pub plan: OperationPlan,
    pub response_blueprint: ResponseBlueprint,

    logical_plan_cache_scopes: id_newtypes::IdToMany<LogicalPlanId, cache_scopes::CacheScopeId>,
    cache_scopes: Vec<cache_scopes::CacheScopeRecord>,
}

impl std::ops::Deref for PreparedOperation {
    type Target = Operation;
    fn deref(&self) -> &Self::Target {
        &self.operation
    }
}

impl std::ops::Index<LogicalPlanId> for PreparedOperation {
    type Output = LogicalPlan;

    fn index(&self, index: LogicalPlanId) -> &Self::Output {
        &self.plan[index]
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

#[derive(Clone, serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectDefinitionId,
    pub root_selection_set_id: BoundSelectionSetId,
    // sorted
    pub root_query_modifier_ids: Vec<BoundQueryModifierId>,
    pub response_keys: ResponseKeys,
    #[indexed_by(BoundSelectionSetId)]
    pub selection_sets: Vec<BoundSelectionSet>,
    #[indexed_by(BoundFieldId)]
    pub fields: Vec<BoundField>,
    #[indexed_by(BoundVariableDefinitionId)]
    pub variable_definitions: Vec<BoundVariableDefinition>,
    #[indexed_by(BoundFieldArgumentId)]
    pub field_arguments: Vec<BoundFieldArgument>,
    pub query_input_values: QueryInputValues,
    // deduplicated by rule
    #[indexed_by(BoundQueryModifierId)]
    pub query_modifiers: Vec<BoundQueryModifier>,
    #[indexed_by(BoundQueryModifierImpactedFieldId)]
    pub query_modifier_impacted_fields: Vec<BoundFieldId>,
    // deduplicated by rule
    #[indexed_by(BoundResponseModifierId)]
    pub response_modifiers: Vec<BoundResponseModifier>,
    #[indexed_by(BoundResponseModifierImpactedFieldId)]
    pub response_modifier_impacted_fields: Vec<BoundFieldId>,
}

#[derive(serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct OperationPlan {
    #[indexed_by(BoundFieldId)]
    pub field_to_logical_plan_id: Vec<LogicalPlanId>,
    pub field_to_solved_requirement: Vec<Option<RequiredFieldId>>,
    pub selection_set_to_objects_must_be_tracked: BitSet<BoundSelectionSetId>,
    #[indexed_by(LogicalPlanId)]
    pub logical_plans: Vec<LogicalPlan>,
    pub mutation_fields_plan_order: Vec<LogicalPlanId>,
    pub children: IdToMany<LogicalPlanId, LogicalPlanId>,
    // LogicalPlanId -> u16
    pub parent_count: Vec<u16>,
    // All dependencies of a plan are placed before it.
    pub in_topological_order: Vec<LogicalPlanId>,
    // Sorted
    pub solved_requirements: Vec<(BoundSelectionSetId, SolvedRequiredFieldSet)>,
}

impl OperationPlan {
    pub fn plan_id_for_field(&self, field_id: BoundFieldId) -> LogicalPlanId {
        self.field_to_logical_plan_id[usize::from(field_id)]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct LogicalPlan {
    pub resolver_id: ResolverDefinitionId,
    pub entity_id: EntityDefinitionId,
    pub root_field_ids_ordered_by_parent_entity_id_then_position: Vec<BoundFieldId>,
}

pub(crate) type SolvedRequiredFieldSet = Vec<SolvedRequiredField>;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct SolvedRequiredField {
    pub id: RequiredFieldId,
    pub field_id: BoundFieldId,
    pub subselection: SolvedRequiredFieldSet,
}

#[derive(serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct ResponseBlueprint {
    pub shapes: Shapes,
    pub field_to_shape_ids: IdToMany<BoundFieldId, FieldShapeId>,
    pub response_modifier_impacted_field_to_response_object_set: Vec<ResponseObjectSetId>,
    #[indexed_by(LogicalPlanId)]
    pub logical_plan_to_blueprint: Vec<LogicalPlanResponseBlueprint>,
    pub selection_set_to_requires_typename: BitSet<BoundSelectionSetId>,
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

    pub fn plan_id_for(&self, id: BoundFieldId) -> LogicalPlanId {
        self.plan.field_to_logical_plan_id[usize::from(id)]
    }

    pub fn solved_requirements_for(&self, id: BoundSelectionSetId) -> Option<&SolvedRequiredFieldSet> {
        self.plan
            .solved_requirements
            .binary_search_by(|probe| probe.0.cmp(&id))
            .map(|ix| &self.plan.solved_requirements[ix].1)
            .ok()
    }
}

impl Operation {
    pub fn walker_with<'op, 'schema>(&'op self, schema: &'schema Schema) -> OperationWalker<'op, ()>
    where
        'schema: 'op,
    {
        OperationWalker {
            schema,
            operation: self,
            item: (),
        }
    }
}

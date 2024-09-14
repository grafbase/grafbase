mod bind;
mod blueprint;
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

/// Represents a GraphQL operation.
///
/// This struct is utilized to hold all necessary information
/// for executing a GraphQL operation including type,
/// associated identifiers, selection sets, and modifiers.
#[derive(serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct Operation {
    /// The type of the operation (query, mutation, etc.).
    pub ty: OperationType,
    /// Identifier for the root object in the operation.
    pub root_object_id: ObjectDefinitionId,
    /// Identifier for the selection set at the root of the operation.
    pub root_selection_set_id: SelectionSetId,
    /// A sorted vector of query modifier identifiers.
    pub root_query_modifier_ids: Vec<QueryModifierId>,
    /// Keys used in the response for the operation.
    pub response_keys: ResponseKeys,
    /// A vector of selection sets associated with the operation.
    #[indexed_by(SelectionSetId)]
    pub selection_sets: Vec<SelectionSet>,
    /// A vector of fields involved in the operation.
    #[indexed_by(FieldId)]
    pub fields: Vec<Field>,
    /// A vector defining variables used in the operation.
    #[indexed_by(VariableDefinitionId)]
    pub variable_definitions: Vec<VariableDefinition>,
    /// A vector of arguments for fields in the operation.
    #[indexed_by(FieldArgumentId)]
    pub field_arguments: Vec<FieldArgument>,
    /// Input values for the query.
    pub query_input_values: QueryInputValues,
    /// A vector of query modifiers applied to the operation, deduplicated by rule.
    #[indexed_by(QueryModifierId)]
    pub query_modifiers: Vec<QueryModifier>,
    /// Fields impacted by the query modifiers.
    #[indexed_by(QueryModifierImpactedFieldId)]
    pub query_modifier_impacted_fields: Vec<FieldId>,
    /// A vector of response modifiers for the operation, deduplicated by rule.
    #[indexed_by(ResponseModifierId)]
    pub response_modifiers: Vec<ResponseModifier>,
    /// Fields impacted by the response modifiers.
    #[indexed_by(ResponseModifierImpactedFieldId)]
    pub response_modifier_impacted_fields: Vec<FieldId>,
}

/// Represents a plan for executing operations within the GraphQL framework.
///
/// This struct holds the mapping between fields and their corresponding logical plans,
/// tracks which selection sets must be monitored, and orders the logical plans based on
/// their dependencies. It also maintains information on solved requirements for the fields
/// processed in the operation.
#[derive(serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct OperationPlan {
    /// A vector that maps each field to its corresponding logical plan ID.
    #[indexed_by(FieldId)]
    pub field_to_logical_plan_id: Vec<LogicalPlanId>,
    /// A vector that holds optional solved requirements associated with each field.
    pub field_to_solved_requirement: Vec<Option<RequiredFieldId>>,
    /// A bitset that indicates which selection sets require tracking.
    pub selection_set_to_objects_must_be_tracked: BitSet<SelectionSetId>,
    /// A vector containing all logical plans associated with the operation.
    #[indexed_by(LogicalPlanId)]
    pub logical_plans: Vec<LogicalPlan>,
    /// A vector representing the order of logical plans specific to mutation fields.
    pub mutation_fields_plan_order: Vec<LogicalPlanId>,
    /// A mapping of each logical plan ID to its children logical plan IDs.
    pub children: IdToMany<LogicalPlanId, LogicalPlanId>,
    /// A vector that counts the number of parents for each logical plan ID.
    pub parent_count: Vec<u16>,
    /// A vector of logical plans ordered topologically based on dependencies.
    pub in_topological_order: Vec<LogicalPlanId>,
    /// A vector of sorted solved requirements, paired with their associated selection set IDs.
    pub solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

impl OperationPlan {
    pub fn plan_id_for_field(&self, field_id: FieldId) -> LogicalPlanId {
        self.field_to_logical_plan_id[usize::from(field_id)]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct LogicalPlan {
    /// The ID of the resolver associated with this logical plan.
    pub resolver_id: ResolverDefinitionId,
    /// The ID of the entity that this logical plan operates on.
    pub entity_id: EntityDefinitionId,
    /// A vector of field IDs ordered by their parent entity ID and their position.
    pub root_field_ids: Vec<FieldId>,
}

pub(crate) type SolvedRequiredFieldSet = Vec<SolvedRequiredField>;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SolvedRequiredField {
    /// The identifier for the required field.
    pub id: RequiredFieldId,
    /// The identifier for the field associated with this requirement.
    pub field_id: FieldId,
    /// A collection of subselections that are also required for this field.
    pub subselection: SolvedRequiredFieldSet,
}

#[derive(serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct ResponseBlueprint {
    /// A collection of shapes used in the response.
    pub shapes: Shapes,
    /// Maps field identifiers to their corresponding shape identifiers.
    pub field_to_shape_ids: IdToMany<FieldId, FieldShapeId>,
    /// A vector of response object sets impacted by response modifiers.
    pub response_modifier_impacted_field_to_response_object_set: Vec<ResponseObjectSetId>,
    /// A vector mapping logical plans to their corresponding blueprints.
    #[indexed_by(LogicalPlanId)]
    pub logical_plan_to_blueprint: Vec<LogicalPlanResponseBlueprint>,
    /// A bitset indicating which selection sets require the inclusion of type names.
    pub selection_set_to_requires_typename: BitSet<SelectionSetId>,
    /// A vector mapping response object sets to their selection set types.
    pub response_object_sets_to_type: Vec<SelectionSetType>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct LogicalPlanResponseBlueprint {
    /// The identifier for the input response object set.
    pub input_id: ResponseObjectSetId,
    /// A range of output response object set identifiers.
    pub output_ids: IdRange<ResponseObjectSetId>,
    /// The identifier for the concrete shape associated with this blueprint.
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
    /// Returns the type of the GraphQL operation (query, mutation, etc.).
    ///
    /// This method provides access to the `OperationType` of the prepared operation,
    /// allowing users to understand what kind of operation is being executed.
    ///
    /// # Returns
    ///
    /// * `OperationType`: The type of the operation.
    pub fn ty(&self) -> OperationType {
        self.operation.ty
    }

    /// Returns the logical plan ID associated with the given field ID.
    ///
    /// This method retrieves the mapping between a field and its corresponding
    /// logical plan ID from the operation's plan. It allows users to identify
    /// which logical plan is responsible for processing a specific field in the
    /// GraphQL operation.
    ///
    /// # Parameters
    ///
    /// * `id`: The identifier of the field for which the logical plan ID is requested.
    ///
    /// # Returns
    ///
    /// * `LogicalPlanId`: The ID of the logical plan associated with the given field ID.
    pub fn plan_id_for(&self, id: FieldId) -> LogicalPlanId {
        self.plan.field_to_logical_plan_id[usize::from(id)]
    }

    /// Returns the solved requirements for the given selection set ID.
    ///
    /// This method searches the operation plan for any solved requirements associated with
    /// the specified selection set. If found, it returns a reference to the set of solved
    /// required fields; otherwise, it returns `None`.
    ///
    /// # Parameters
    ///
    /// * `id`: The identifier of the selection set for which the solved requirements are requested.
    ///
    /// # Returns
    ///
    /// * `Option<&SolvedRequiredFieldSet>`: An optional reference to the set of solved required fields,
    ///   or `None` if no requirements are found for the given selection set ID.
    pub fn solved_requirements_for(&self, id: SelectionSetId) -> Option<&SolvedRequiredFieldSet> {
        self.plan
            .solved_requirements
            .binary_search_by(|probe| probe.0.cmp(&id))
            .map(|ix| &self.plan.solved_requirements[ix].1)
            .ok()
    }
}

impl Operation {
    /// Creates a walker for traversing the GraphQL operation.
    ///
    /// # Parameters
    ///
    /// * `schema`: A reference to the schema that defines the operation context.
    ///
    /// # Returns
    ///
    /// * `OperationWalker<'op, ()>`: A walker that can be used to traverse the operation.
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

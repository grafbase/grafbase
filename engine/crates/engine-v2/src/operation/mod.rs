mod bind;
mod build;
mod condition;
pub mod ids;
mod input_value;
mod location;
mod parse;
mod path;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use crate::response::ResponseKeys;
pub(crate) use condition::*;
pub(crate) use engine_parser::types::OperationType;
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use path::QueryPath;
use schema::{ObjectId, RequiredFieldId, ResolverId, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

pub(crate) struct Plan {
    pub resolver_id: ResolverId,
}

pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    pub root_condition_id: Option<ConditionId>,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub response_keys: ResponseKeys,
    pub root_selection_set_id: SelectionSetId,
    pub selection_sets: Vec<SelectionSet>,
    pub fields: Vec<Field>,
    pub field_to_parent: Vec<SelectionSetId>,
    pub fragments: Vec<Fragment>,
    pub fragment_spreads: Vec<FragmentSpread>,
    pub inline_fragments: Vec<InlineFragment>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub field_arguments: Vec<FieldArgument>,
    pub query_input_values: QueryInputValues,
    // deduplicated
    pub conditions: Vec<Condition>,
    // -- Added during planning --
    pub plans: Vec<Plan>,
    // Sorted
    pub plan_edges: Vec<ParentToChildEdge>,
    pub field_dependencies: Vec<FieldDependency>,
    pub field_to_plan_id: Vec<Option<PlanId>>,
    pub selection_set_to_plan_id: Vec<Option<PlanId>>,
    pub field_to_entity_location: Vec<Option<EntityLocation>>,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParentToChildEdge {
    pub parent: PlanId,
    pub child: PlanId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldDependency {
    pub entity_location: EntityLocation,
    pub required_field_id: RequiredFieldId,
    pub field_id: FieldId,
}

impl Operation {
    pub fn parent_selection_set_id(&self, id: FieldId) -> SelectionSetId {
        self.field_to_parent[usize::from(id)]
    }

    pub fn walker_with<'op, 'schema, SI>(
        &'op self,
        schema_walker: SchemaWalker<'schema, SI>,
        variables: &'op Variables,
    ) -> OperationWalker<'op, (), SI>
    where
        'schema: 'op,
    {
        OperationWalker {
            operation: self,
            variables,
            schema_walker,
            item: (),
        }
    }

    pub fn find_matching_field(
        &self,
        entity_location: EntityLocation,
        required_field_id: RequiredFieldId,
    ) -> Option<FieldId> {
        self.field_dependencies
            .binary_search_by(|field_dependency| {
                field_dependency
                    .entity_location
                    .cmp(&entity_location)
                    .then(field_dependency.required_field_id.cmp(&required_field_id))
            })
            .ok()
            .map(|index| self.field_dependencies[index].field_id)
    }
}

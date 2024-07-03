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

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Plan {
    pub resolver_id: ResolverId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OperationMetadata {
    pub ty: OperationType,
    /// This is a *processed* operation name, it does not strictly reflect the GraphQL operation
    /// name. Currently, if the latter doesn't exist we take the first field's name as the
    /// operation name.
    pub name: Option<String>,
    pub normalized_query: String,
    pub normalized_query_hash: [u8; 32],
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Operation {
    pub metadata: OperationMetadata,
    pub root_object_id: ObjectId,
    pub root_condition_id: Option<ConditionId>,
    pub root_selection_set_id: SelectionSetId,
    pub response_keys: ResponseKeys,
    pub selection_sets: Vec<SelectionSet>,
    pub fields: Vec<Field>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub field_arguments: Vec<FieldArgument>,
    pub query_input_values: QueryInputValues,
    // deduplicated
    pub conditions: Vec<Condition>,
    // -- Added during planning --
    pub plans: Vec<Plan>,
    pub field_to_plan_id: Vec<PlanId>,
    // Sorted
    pub plan_edges: Vec<ParentToChildEdge>,
    pub solved_requirements: Vec<(SelectionSetId, SolvedRequiredFieldSet)>,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) struct ParentToChildEdge {
    pub parent: PlanId,
    pub child: PlanId,
}

pub(crate) type SolvedRequiredFieldSet = Vec<SolvedRequiredField>;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SolvedRequiredField {
    pub id: RequiredFieldId,
    pub field_id: FieldId,
    pub subselection: SolvedRequiredFieldSet,
}

impl Operation {
    pub fn plan_id_for(&self, id: FieldId) -> PlanId {
        self.field_to_plan_id[usize::from(id)]
    }

    pub fn solved_requirements_for(&self, id: SelectionSetId) -> Option<&SolvedRequiredFieldSet> {
        self.solved_requirements
            .binary_search_by(|probe| probe.0.cmp(&id))
            .map(|ix| &self.solved_requirements[ix].1)
            .ok()
    }

    pub fn ty(&self) -> OperationType {
        self.metadata.ty
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
}

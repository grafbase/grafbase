mod ids;
mod modifier;
mod selection_set;
mod variables;

use grafbase_telemetry::graphql::OperationType;
use id_derives::IndexedFields;
pub(crate) use ids::*;
pub(crate) use modifier::*;
use schema::{ObjectDefinitionId, Schema};
pub(crate) use selection_set::*;
pub(crate) use variables::*;

use crate::{
    operation::{OperationWalker, QueryInputValues},
    response::ResponseKeys,
};

#[derive(Clone, serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct BoundOperation {
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

impl BoundOperation {
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

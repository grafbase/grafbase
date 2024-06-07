mod bind;
mod build;
mod cache_control;
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
pub use cache_control::OperationCacheControl;
pub(crate) use engine_parser::types::OperationType;
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use path::QueryPath;
use schema::{ObjectId, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

#[derive(Clone)]
pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
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
    pub cache_control: Option<OperationCacheControl>,
    pub field_arguments: Vec<FieldArgument>,
    pub query_input_values: QueryInputValues,
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
}

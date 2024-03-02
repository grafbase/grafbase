pub(crate) use bind::bind_variables;
pub(crate) use engine_parser::types::OperationType;
pub(crate) use ids::*;
pub(crate) use input_value::*;
pub(crate) use location::Location;
pub(crate) use path::QueryPath;
use runtime::cache::OperationCacheControl;
use schema::{ObjectId, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variable::VariableDefinition;
pub(crate) use walkers::*;

use crate::response::ResponseKeys;

mod bind;
mod build;
pub mod ids;
mod input_value;
mod location;
mod parse;
mod path;
mod selection_set;
mod validation;
mod variable;
mod walkers;

pub(crate) struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    pub name: Option<String>,
    pub response_keys: ResponseKeys,
    pub root_selection_set_id: BoundSelectionSetId,
    pub selection_sets: Vec<BoundSelectionSet>,
    pub fields: Vec<BoundField>,
    pub field_to_parent: Vec<BoundSelectionSetId>,
    pub fragments: Vec<BoundFragment>,
    pub fragment_spreads: Vec<BoundFragmentSpread>,
    pub inline_fragments: Vec<BoundInlineFragment>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub cache_control: Option<OperationCacheControl>,
    pub field_arguments: Vec<BoundFieldArgument>,
    pub input_values: OpInputValues,
}

impl Operation {
    pub fn is_query(&self) -> bool {
        matches!(self.ty, OperationType::Query)
    }

    pub fn parent_selection_set_id(&self, id: BoundFieldId) -> BoundSelectionSetId {
        self.field_to_parent[usize::from(id)]
    }

    pub fn walk_selection_set<'op, 'schema>(
        &'op self,
        schema_walker: SchemaWalker<'schema, ()>,
    ) -> BoundSelectionSetWalker<'op>
    where
        'schema: 'op,
    {
        self.walker_with(schema_walker).walk(self.root_selection_set_id)
    }

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

use std::sync::Arc;

pub use bind::BindResult;
pub use engine_parser::{types::OperationType, Pos};
pub use flat::*;
pub use ids::*;
pub use parse::{parse_operation, UnboundOperation};
pub use path::QueryPath;
use schema::{CacheConfig, Merge, ObjectId, Schema, SchemaWalker};
pub use selection_set::*;
pub use variable::VariableDefinition;
pub use walkers::*;

use crate::response::ResponseKeys;

mod bind;
mod flat;
pub mod ids;
mod parse;
mod path;
mod selection_set;
mod variable;
mod walkers;

pub struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    pub name: Option<String>,
    pub root_selection_set_id: BoundSelectionSetId,
    pub selection_sets: Vec<BoundSelectionSet>,
    pub fields: Vec<BoundField>,
    pub response_keys: Arc<ResponseKeys>,
    pub fragment_definitions: Vec<BoundFragmentDefinition>,
    pub field_definitions: Vec<BoundAnyFieldDefinition>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub cache_config: Option<CacheConfig>,
}

impl Operation {
    /// Builds an `Operation` by binding unbound operation to a schema and configuring its non functional requirements
    /// like caching, auth, ....
    ///
    /// All field names are mapped to their actual field id in the schema and respective configuration.
    /// At this stage the operation might not be resolvable but it should make sense given the schema types.
    pub fn build(schema: &Schema, unbound_operation: UnboundOperation) -> BindResult<Self> {
        let mut operation = Self::bind(schema, unbound_operation)?;

        if operation.ty == OperationType::Query {
            let root_cache_config = schema[operation.root_object_id]
                .cache_config
                .map(|cache_config_id| schema[cache_config_id]);

            let selection_set_cache_config = operation
                .walker_with(schema.walker())
                .walk(operation.root_selection_set_id)
                .cache_config();

            operation.cache_config = root_cache_config.merge(selection_set_cache_config);
        }

        Ok(operation)
    }

    fn bind(schema: &Schema, unbound_operation: UnboundOperation) -> BindResult<Self> {
        bind::bind(schema, unbound_operation)
    }

    pub fn walker_with<'op, 'schema>(
        &'op self,
        schema_walker: SchemaWalker<'schema, ()>,
    ) -> OperationWalker<'op, (), (), ()>
    where
        'schema: 'op,
    {
        OperationWalker {
            operation: self,
            schema_walker,
            ctx: (),
            item: (),
        }
    }
}

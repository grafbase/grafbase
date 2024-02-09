pub(crate) use bind::BindResult;
pub(crate) use engine_parser::types::OperationType;
pub(crate) use flat::*;
pub(crate) use ids::*;
pub(crate) use location::Location;
pub(crate) use parse::{parse_operation, ParsedOperation};
pub(crate) use path::QueryPath;
use schema::{CacheConfig, Merge, ObjectId, Schema, SchemaWalker};
pub(crate) use selection_set::*;
pub(crate) use variable::VariableDefinition;
pub(crate) use walkers::*;

use crate::response::ResponseKeys;

use self::bind::{BindError, OperationLimitExceededError};

mod bind;
mod flat;
pub mod ids;
mod location;
mod parse;
mod path;
mod selection_set;
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
    pub cache_config: Option<CacheConfig>,
    pub field_arguments: Vec<BoundFieldArguments>,
}

pub type BoundFieldArguments = Vec<BoundFieldArgument>;

impl Operation {
    pub fn parent_selection_set_id(&self, id: BoundFieldId) -> BoundSelectionSetId {
        self.field_to_parent[usize::from(id)]
    }

    pub fn empty_arguments(&self) -> &BoundFieldArguments {
        &self.field_arguments[0]
    }

    fn enforce_operation_limits(&self, schema: &Schema) -> Result<(), OperationLimitExceededError> {
        let selection_set = self.walker_with(schema.walker()).walk(self.root_selection_set_id);

        if let Some(depth_limit) = schema.operation_limits.depth {
            let max_depth = selection_set.max_depth();
            if max_depth > depth_limit {
                return Err(OperationLimitExceededError::QueryTooDeep);
            }
        }

        if let Some(max_alias_count) = schema.operation_limits.aliases {
            let alias_count = selection_set.alias_count();
            if alias_count > max_alias_count {
                return Err(OperationLimitExceededError::QueryContainsTooManyAliases);
            }
        }

        if let Some(max_root_field_count) = schema.operation_limits.root_fields {
            let root_field_count = selection_set.root_field_count();
            if root_field_count > max_root_field_count {
                return Err(OperationLimitExceededError::QueryContainsTooManyRootFields);
            }
        }

        if let Some(max_height) = schema.operation_limits.height {
            let height = selection_set.height(&mut Default::default());
            if height > max_height {
                return Err(OperationLimitExceededError::QueryTooHigh);
            }
        }

        if let Some(max_complexity) = schema.operation_limits.complexity {
            let complexity = selection_set.complexity();
            if complexity > max_complexity {
                return Err(OperationLimitExceededError::QueryTooComplex);
            }
        }

        Ok(())
    }

    /// Builds an `Operation` by binding unbound operation to a schema and configuring its non functional requirements
    /// like caching, auth, ....
    ///
    /// All field names are mapped to their actual field id in the schema and respective configuration.
    /// At this stage the operation might not be resolvable but it should make sense given the schema types.
    pub fn build(
        schema: &Schema,
        unbound_operation: ParsedOperation,
        operation_limits_enabled: bool,
    ) -> BindResult<Self> {
        let mut operation = bind::bind(schema, unbound_operation)?;

        if operation_limits_enabled {
            operation
                .enforce_operation_limits(schema)
                .map_err(BindError::OperationLimitExceeded)?;
        }

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

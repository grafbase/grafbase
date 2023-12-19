use std::sync::Arc;

pub use bind::BindResult;
pub use engine_parser::{types::OperationType, Pos};
pub use flat::*;
pub use ids::*;
pub use parse::{parse_operation, UnboundOperation};
pub use path::QueryPath;
use schema::{CacheConfig, Definition, Merge, ObjectId, Schema, SchemaWalker};
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

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

            let selection_cache_config = operation
                .walker_with(schema.walker(), ())
                .walk(operation.root_selection_set_id)
                .iter()
                .filter_map(|bound_selection| {
                    Self::traverse_bound_selection_for_caching(schema, &operation, bound_selection)
                })
                .reduce(|a, b| a.merge(b));

            operation.cache_config = root_cache_config.merge(selection_cache_config);
        }

        Ok(operation)
    }

    fn bind(schema: &Schema, unbound_operation: UnboundOperation) -> BindResult<Self> {
        bind::bind(schema, unbound_operation)
    }

    // this merely traverses the selection set recursively and merge all cache_config present in the
    // selected fields
    fn traverse_bound_selection_for_caching(
        schema: &Schema,
        operation: &Operation,
        bound_selection: &BoundSelection,
    ) -> Option<CacheConfig> {
        match bound_selection {
            BoundSelection::Field(bounded_field_id) => {
                let bound_field = operation[*bounded_field_id];
                let bound_field_definition = &operation[bound_field.definition_id];

                let field_cache_config: Option<CacheConfig> =
                    if let BoundAnyFieldDefinition::Field(field) = bound_field_definition {
                        let field_walker = schema.walker().walk(field.field_id);
                        let field_cache_config = field_walker.cache_config();
                        let field_type = &schema[field_walker.type_id];

                        match field_type.inner {
                            Definition::Object(object_id) => {
                                let object = &schema[object_id];
                                let object_cache_config =
                                    object.cache_config.map(|cache_control_id| schema[cache_control_id]);

                                object_cache_config.merge(field_cache_config)
                            }
                            _ => field_cache_config,
                        }
                    } else {
                        None
                    };

                if let Some(selection_set_id) = bound_field.selection_set_id {
                    let selection_set_cache_config = operation[selection_set_id]
                        .items
                        .iter()
                        .filter_map(|bound_selection| {
                            Self::traverse_bound_selection_for_caching(schema, operation, bound_selection)
                        })
                        .reduce(|a, b| a.merge(b));

                    field_cache_config.merge(selection_set_cache_config)
                } else {
                    field_cache_config
                }
            }
            BoundSelection::FragmentSpread(spread) => {
                let selection_set = &operation[spread.selection_set_id];

                selection_set
                    .items
                    .iter()
                    .filter_map(|bound_selection| {
                        Self::traverse_bound_selection_for_caching(schema, operation, bound_selection)
                    })
                    .reduce(|a, b| a.merge(b))
            }
            BoundSelection::InlineFragment(inline) => {
                let selection_set = &operation[inline.selection_set_id];

                selection_set
                    .items
                    .iter()
                    .filter_map(|bound_selection| {
                        Self::traverse_bound_selection_for_caching(schema, operation, bound_selection)
                    })
                    .reduce(|a, b| a.merge(b))
            }
        }
    }

    pub fn walker_with<'op, 'schema, E>(
        &'op self,
        schema_walker: SchemaWalker<'schema, ()>,
        ext: E,
    ) -> OperationWalker<'op, (), (), E>
    where
        'schema: 'op,
    {
        OperationWalker {
            operation: self,
            schema_walker,
            ext,
            wrapped: (),
        }
    }
}

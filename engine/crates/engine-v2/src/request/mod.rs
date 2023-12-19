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

            let selection_set_cache_config = operation
                .walker_with(schema.walker(), ())
                .walk(operation.root_selection_set_id)
                .into_iter()
                .filter_map(|bound_selection| Self::traverse_bound_selection_for_caching(schema, bound_selection))
                .reduce(|a, b| a.merge(b));

            operation.cache_config = root_cache_config.merge(selection_set_cache_config);
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
        bound_selection_walker: BoundSelectionWalker<'_>,
    ) -> Option<CacheConfig> {
        match bound_selection_walker {
            BoundSelectionWalker::Field(bounded_field_walker) => {
                let bounded_field_definition = bounded_field_walker.definition().get();

                let bounded_field_cache_config = match bounded_field_definition {
                    BoundAnyFieldDefinition::Field(bounded_field_definition) => {
                        let field_walker = schema.walker().walk(bounded_field_definition.field_id);
                        let field_cache_config = field_walker.cache_config();
                        let field_type = field_walker.ty().get();

                        let object_field_cache_config = match field_type.inner {
                            Definition::Object(object_id) => {
                                let object = field_walker.walk(object_id).get();
                                let object_cache_config =
                                    object.cache_config.map(|cache_control_id| schema[cache_control_id]);

                                object_cache_config.merge(field_cache_config)
                            }
                            _ => None,
                        };

                        field_cache_config.merge(object_field_cache_config)
                    }
                    BoundAnyFieldDefinition::TypeName(_) => None,
                };

                let bounded_field_selection_set_cache_config =
                    bounded_field_walker.selection_set().and_then(|selection_set_walker| {
                        let selection_set_cache_config = selection_set_walker
                            .into_iter()
                            .filter_map(|bound_selection| {
                                Self::traverse_bound_selection_for_caching(schema, bound_selection)
                            })
                            .reduce(|a, b| a.merge(b));

                        bounded_field_cache_config.merge(selection_set_cache_config)
                    });

                bounded_field_cache_config.merge(bounded_field_selection_set_cache_config)
            }
            BoundSelectionWalker::InlineFragment(inline) => inline
                .selection_set()
                .into_iter()
                .filter_map(|bound_selection| Self::traverse_bound_selection_for_caching(schema, bound_selection))
                .reduce(|a, b| a.merge(b)),
            BoundSelectionWalker::FragmentSpread(spread) => spread
                .selection_set()
                .into_iter()
                .filter_map(|bound_selection| Self::traverse_bound_selection_for_caching(schema, bound_selection))
                .reduce(|a, b| a.merge(b)),
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

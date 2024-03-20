use schema::{CacheConfig, Merge, Schema};

use crate::response::GraphqlError;

use super::{BoundSelectionSetWalker, Operation, OperationCacheControl, OperationWalker, Variables};

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error(transparent)]
    Bind(#[from] super::bind::BindError),
    #[error(transparent)]
    Validation(#[from] super::validation::ValidationError),
    #[error(transparent)]
    Parse(#[from] super::parse::ParseError),
}

impl From<OperationError> for GraphqlError {
    fn from(err: OperationError) -> Self {
        match err {
            OperationError::Bind(err) => err.into(),
            OperationError::Validation(err) => err.into(),
            OperationError::Parse(err) => err.into(),
        }
    }
}

impl Operation {
    /// Builds an `Operation` by binding unbound operation to a schema and configuring its non functional requirements
    /// like caching, auth, ....
    ///
    /// All field names are mapped to their actual field id in the schema and respective configuration.
    /// At this stage the operation might not be resolvable but it should make sense given the schema types.
    pub fn build(schema: &Schema, request: &engine::Request) -> Result<Self, OperationError> {
        let parsed_operation = super::parse::parse_operation(request)?;
        let mut operation = super::bind::bind(schema, parsed_operation)?;

        // Creating a walker with no variables enabling validation to use them
        let variables = Variables::empty_for(&operation);
        operation.cache_control = compute_cache_control(operation.walker_with(schema.walker(), &variables), request);
        super::validation::validate_operation(schema, operation.walker_with(schema.walker(), &variables), request)?;

        Ok(operation)
    }
}

fn compute_cache_control(operation: OperationWalker<'_>, request: &engine::Request) -> Option<OperationCacheControl> {
    if operation.is_query() {
        let root_cache_config = operation.root_object().cache_config();
        let selection_set = operation.selection_set();

        let selection_set_cache_config = selection_set.cache_config();
        root_cache_config.merge(selection_set_cache_config).map(
            |CacheConfig {
                 max_age,
                 stale_while_revalidate,
             }| OperationCacheControl {
                max_age,
                key: request.cache_key(),
                stale_while_revalidate,
            },
        )
    } else {
        None
    }
}

impl BoundSelectionSetWalker<'_> {
    // this merely traverses the selection set recursively and merge all cache_config present in the
    // selected fields
    fn cache_config(&self) -> Option<CacheConfig> {
        self.fields()
            .filter_map(|field| {
                let cache_config = field.schema_field().and_then(|definition| {
                    definition
                        .cache_config()
                        .merge(definition.ty().inner().as_object().and_then(|obj| obj.cache_config()))
                });
                let selection_set_cache_config = field
                    .selection_set()
                    .and_then(|selection_set| selection_set.cache_config());
                cache_config.merge(selection_set_cache_config)
            })
            .reduce(|a, b| a.merge(b))
    }
}

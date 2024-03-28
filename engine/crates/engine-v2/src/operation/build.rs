use schema::CacheControl;

use crate::{execution::ExecutionContext, response::GraphqlError};

use super::{Operation, OperationCacheControl, OperationWalker, SelectionSetWalker, Variables};

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
    pub fn build(ctx: ExecutionContext<'_>, request: &engine::Request) -> Result<Self, OperationError> {
        let schema = &ctx.engine.schema;
        let parsed_operation = super::parse::parse_operation(request)?;
        let mut operation = super::bind::bind(schema, parsed_operation)?;

        // Creating a walker with no variables enabling validation to use them
        let variables = Variables::empty_for(&operation);
        operation.cache_control = compute_cache_control(operation.walker_with(schema.walker(), &variables), request);
        super::validation::validate_operation(ctx, operation.walker_with(schema.walker(), &variables), request)?;

        Ok(operation)
    }
}

fn compute_cache_control(operation: OperationWalker<'_>, request: &engine::Request) -> Option<OperationCacheControl> {
    if operation.is_query() {
        let root_cache_control = operation.root_object().directives().cache_control();
        let selection_set = operation.selection_set();

        let selection_set_cache_config = selection_set.computed_cache_control();
        CacheControl::union_opt(root_cache_control, selection_set_cache_config.as_ref()).map(
            |CacheControl {
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

impl SelectionSetWalker<'_> {
    // this merely traverses the selection set recursively and merge all cache_control present in the
    // selected fields
    fn computed_cache_control(&self) -> Option<CacheControl> {
        self.fields()
            .filter_map(|field| {
                let cache_control = field.definition().and_then(|definition| {
                    CacheControl::union_opt(
                        definition.directives().cache_control(),
                        definition.ty().inner().directives().cache_control(),
                    )
                });
                CacheControl::union_opt(
                    cache_control.as_ref(),
                    field.selection_set().and_then(|s| s.computed_cache_control()).as_ref(),
                )
            })
            .reduce(|a, b| a.union(b))
    }
}

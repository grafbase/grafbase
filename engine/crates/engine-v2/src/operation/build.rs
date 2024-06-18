use grafbase_tracing::{gql_response_status::GraphqlResponseStatus, metrics::GraphqlOperationMetricsAttributes};
use schema::CacheControl;

use crate::{execution::ExecutionContext, response::GraphqlError};

use super::{Operation, OperationCacheControl, OperationWalker, SelectionSetWalker, Variables};

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error(transparent)]
    Parse(#[from] super::parse::ParseError),
    #[error("{err}")]
    Bind {
        operation_attributes: Box<Option<GraphqlOperationMetricsAttributes>>,
        err: super::bind::BindError,
    },
    #[error("{err}")]
    Validation {
        operation_attributes: Box<Option<GraphqlOperationMetricsAttributes>>,
        err: super::validation::ValidationError,
    },
}

impl From<OperationError> for GraphqlError {
    fn from(err: OperationError) -> Self {
        match err {
            OperationError::Bind { err, .. } => err.into(),
            OperationError::Validation { err, .. } => err.into(),
            OperationError::Parse(err) => err.into(),
        }
    }
}

impl OperationError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationMetricsAttributes> {
        match self {
            OperationError::Bind {
                operation_attributes, ..
            } => std::mem::take(operation_attributes),
            OperationError::Validation {
                operation_attributes, ..
            } => std::mem::take(operation_attributes),
            _ => None,
        }
    }
}

impl Operation {
    /// Builds an `Operation` by binding unbound operation to a schema and configuring its non functional requirements
    /// like caching, auth, ....
    ///
    /// All field names are mapped to their actual field id in the schema and respective configuration.
    /// At this stage the operation might not be resolvable but it should make sense given the schema types.
    pub fn build(
        ctx: ExecutionContext<'_>,
        request: &engine::Request,
    ) -> Result<(Self, Option<GraphqlOperationMetricsAttributes>), OperationError> {
        let schema = &ctx.engine.schema;
        let parsed_operation = super::parse::parse_operation(request)?;
        let operation_attributes = operation_normalizer::normalize(request.query(), request.operation_name())
            .ok()
            .map(|normalized_query| GraphqlOperationMetricsAttributes {
                normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                name: parsed_operation.name.clone(),
                ty: parsed_operation.definition.ty.as_str(),
                normalized_query,
                // overridden at the end.
                status: GraphqlResponseStatus::Success,
                cache_status: None,
                client: ctx.request_metadata.client.clone(),
            });

        let mut operation = match super::bind::bind(schema, parsed_operation) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(OperationError::Bind {
                    operation_attributes: Box::new(operation_attributes),
                    err,
                })
            }
        };

        // Creating a walker with no variables enabling validation to use them
        let variables = Variables::empty_for(&operation);
        operation.cache_control = compute_cache_control(operation.walker_with(schema.walker(), &variables), request);
        if let Err(err) =
            super::validation::validate_operation(ctx, operation.walker_with(schema.walker(), &variables), request)
        {
            return Err(OperationError::Validation {
                operation_attributes: Box::new(operation_attributes),
                err,
            });
        }

        Ok((operation, operation_attributes))
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

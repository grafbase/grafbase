use std::{future::Future, sync::Arc};

use engine_schema::{ExtensionDirective, FieldDefinition};
use error::GraphqlError;
use futures_util::stream::BoxStream;

use crate::hooks::Anything;

use super::Data;

pub trait FieldResolverExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    /// The data will be cached as part of the operation plan. Beware that the cache key will be
    /// depended on the schema & extensions, but not on the configuration.
    fn prepare<'ctx>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: impl Anything<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    /// Resolve a field through an extension. Lifetime 'ctx will be available for as long as the
    /// future lives, but 'resp lifetime won't. It provides access to the response data that is
    /// shared, without lock, so it's only temporarily available.
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription_field<'ctx, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;
}

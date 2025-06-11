use std::future::Future;

use engine_schema::{ExtensionDirective, FieldDefinition};
use error::GraphqlError;
use futures_util::stream::BoxStream;

use crate::hooks::Anything;

use super::Data;

pub trait FieldResolverExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    /// Resolve a field through an extension. Lifetime 'ctx will be available for as long as the
    /// future lives, but 'resp lifetime won't. It provides access to the response data that is
    /// shared, without lock, so it's only temporarily available.
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
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
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;
}

mod authorization;
mod resolver;
mod token;

use std::{future::Future, ops::Range, sync::Arc};

use engine_schema::{DirectiveSite, ExtensionDirective, FieldDefinition};
use extension_catalog::ExtensionId;
use futures_util::stream::BoxStream;

use crate::hooks::Anything;
use error::{ErrorResponse, GraphqlError};

pub use authorization::*;
pub use resolver::*;
pub use token::*;

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime: Send + Sync + 'static {
    type SharedContext: Send + Sync + 'static;

    fn prepare_field<'ctx>(
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

    fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn authenticate(
        &self,
        extension_ids: &[ExtensionId],
        gateway_headers: http::HeaderMap,
    ) -> impl Future<Output = (http::HeaderMap, Result<Token, ErrorResponse>)> + Send;

    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        wasm_context: &'ctx Self::SharedContext,
        subgraph_headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<QueryAuthorizationDecisions>), ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
        Arguments: Anything<'ctx>;

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut;
}

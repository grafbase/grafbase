use engine_error::{ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{WasmContext, extension::api::wit, resources::Lease};

pub(crate) type QueryAuthorizationResult =
    wasmtime::Result<Result<(Lease<http::HeaderMap>, AuthorizationDecisions, Vec<u8>), ErrorResponse>>;

pub(crate) trait AuthorizationExtensionInstance {
    fn authorize_query<'a>(
        &'a mut self,
        context: &'a WasmContext,
        headers: Lease<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: wit::QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult>;

    fn authorize_response<'a>(
        &'a mut self,
        context: &'a WasmContext,
        state: &'a [u8],
        elements: wit::ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>>;
}

use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{extension::api::wit, resources::Headers};

pub(crate) trait AuthorizationExtensionInstance {
    fn authorize_query<'a>(
        &'a mut self,
        ctx: EngineRequestContext,
        headers: Headers,
        token: TokenRef<'a>,
        elements: wit::QueryElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizeQueryOutput, ErrorResponse>>>;

    fn authorize_response<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        state: &'a [u8],
        elements: wit::ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>>;
}

pub(crate) struct AuthorizeQueryOutput {
    pub subgraph_headers: Headers,
    pub additional_headers: Option<http::HeaderMap>,
    pub decisions: AuthorizationDecisions,
    pub context: Vec<u8>,
    pub state: Vec<u8>,
}

use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    Error, ErrorResponse,
    extension::api::wit::{QueryElements, ResponseElements},
    resources::Lease,
};

pub(crate) type QueryAuthorizationResult =
    Result<(Lease<http::HeaderMap>, AuthorizationDecisions, Vec<u8>), ErrorResponse>;

pub(crate) trait AuthorizationExtensionInstance {
    fn authorize_query<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult>;

    fn authorize_response<'a>(
        &'a mut self,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>>;
}

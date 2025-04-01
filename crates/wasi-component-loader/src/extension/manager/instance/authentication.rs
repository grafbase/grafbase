use futures::future::BoxFuture;
use runtime::extension::Token;

use crate::{ErrorResponse, resources::Lease};

pub(crate) trait AuthenticationExtensionInstance {
    fn authenticate(
        &mut self,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>>;
}

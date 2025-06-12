use futures::future::BoxFuture;
use runtime::extension::Token;

use crate::{ErrorResponse, SharedContext, resources::Lease};

pub(crate) trait AuthenticationExtensionInstance {
    fn authenticate(
        &mut self,
        context: SharedContext,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>>;
}

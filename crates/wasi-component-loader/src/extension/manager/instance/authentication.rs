use futures::future::BoxFuture;
use runtime::{authentication::PublicMetadataEndpoint, extension::Token};

use crate::{Error, ErrorResponse, SharedContext, resources::Lease};

pub(crate) trait AuthenticationExtensionInstance {
    fn authenticate(
        &mut self,
        context: SharedContext,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>>;

    fn public_metadata(&mut self) -> BoxFuture<'_, Result<Vec<PublicMetadataEndpoint>, Error>> {
        Box::pin(std::future::ready(Ok(vec![])))
    }
}

use engine_error::ErrorResponse;
use futures::future::BoxFuture;
use runtime::extension::{PublicMetadataEndpoint, Token};

use crate::{WasmContext, resources::Headers};

pub(crate) trait AuthenticationExtensionInstance {
    #[allow(clippy::type_complexity)]
    fn authenticate<'a>(
        &'a mut self,
        context: &'a WasmContext,
        headers: Headers,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(Headers, Token), ErrorResponse>>>;

    fn public_metadata(&mut self) -> BoxFuture<'_, wasmtime::Result<Result<Vec<PublicMetadataEndpoint>, String>>> {
        Box::pin(std::future::ready(Ok(Ok(vec![]))))
    }
}

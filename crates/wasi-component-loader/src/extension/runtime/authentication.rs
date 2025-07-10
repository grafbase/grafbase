use std::sync::Arc;

use crate::{SharedContext, extension::EngineWasmExtensions, resources::Lease};
use engine_error::{ErrorCode, ErrorResponse};
use extension_catalog::ExtensionId;
use futures::{StreamExt as _, stream::FuturesUnordered};
use runtime::extension::{AuthenticationExtension, Token};

impl AuthenticationExtension<SharedContext> for EngineWasmExtensions {
    async fn authenticate(
        &self,
        context: &SharedContext,
        extension_ids: &[ExtensionId],
        gateway_headers: http::HeaderMap,
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        assert!(!extension_ids.is_empty(), "At least one extension must be provided");

        let headers = Arc::new(gateway_headers);

        let mut futures = extension_ids
            .iter()
            .map(|id| async {
                let mut instance = self.get(*id).await?;

                instance
                    .authenticate(context.clone(), Lease::Shared(headers.clone()))
                    .await
                    .map(|(_, token)| token)
                    .map_err(|err| err.into_graphql_error_response(ErrorCode::Unauthenticated))
            })
            .collect::<FuturesUnordered<_>>();

        let mut result = futures.next().await.unwrap();

        // In pure Rust, we would cancel all the remaining futures as soon as we retrieve the first token.
        // The problem with Wasm is that while we can cancel the futures, this results in poisoned
        // instances we can't re-use and forces us to re-create new ones.
        //
        // This is likely to be worse than letting the extension run to their end. The
        // authentication logic is already expected to be ran at every request and is written with
        // that in mind. However, re-creating an instance is costly on the host side but also runs
        // the initialization logic which no extension developer expects to be ran at every
        // request, which may lead to unexpected behaviors.
        //
        // Once Wasm provides better future support we might reconsider this decision.
        while let Some(next_result) = futures.next().await {
            result = match (result, next_result) {
                // Take the token if there is any.
                (Ok(token), _) => Ok(token),
                (_, Ok(token)) => Ok(token),
                // If there is a client error, we use it. Server error are likely to be logged and
                // be less useful for clients.
                (Err(err), _) if err.status.is_client_error() => Err(err),
                (_, Err(err)) if err.status.is_client_error() => Err(err),
                (err, _) => err,
            };
        }
        drop(futures);

        (Arc::into_inner(headers).unwrap(), result)
    }

    async fn public_metadata(
        &self,
        extension_ids: &[ExtensionId],
    ) -> Result<Vec<runtime::authentication::PublicMetadataEndpoint>, String> {
        let mut endpoints = Vec::new();

        for id in extension_ids {
            let mut instance = self.get(*id).await.unwrap();

            let mut new_endpoints = instance.public_metadata().await.map_err(|err| err.to_string())?;
            endpoints.append(&mut new_endpoints);
        }

        Ok(endpoints)
    }
}

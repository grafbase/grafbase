use std::sync::Arc;

use crate::{extension::GatewayWasmExtensions, wasmsafe};
use engine_error::ErrorResponse;
use extension_catalog::ExtensionId;
use futures::{StreamExt as _, TryStreamExt as _, stream::FuturesUnordered};
use runtime::extension::{AuthenticationExtension, ExtensionRequestContext, PublicMetadataEndpoint, Token};

impl AuthenticationExtension for GatewayWasmExtensions {
    async fn authenticate(
        &self,
        context: Arc<ExtensionRequestContext>,
        gateway_headers: http::HeaderMap,
        ids: &[ExtensionId],
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let headers = Arc::new(gateway_headers);

        let mut futures = self
            .authentication
            .iter()
            .filter(|pool| ids.contains(&pool.id()))
            .map(|pool| async {
                let mut instance = pool.get().await.map_err(|err| {
                    tracing::error!("Failed to get authentication instance: {err}");
                    ErrorResponse::internal_extension_error()
                })?;
                wasmsafe!(instance.authenticate(context.clone(), headers.clone().into()).await)
            })
            .collect::<FuturesUnordered<_>>()
            .map(|result| match result {
                Ok((_, token)) => Ok(token),
                Err(err) => Err(err),
            });

        let result = if let Some(mut result) = futures.next().await {
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
                    (Ok(token), _) | (_, Ok(token)) => Ok(token),
                    // If there is a client error, we use it. Server error are likely to be logged and
                    // be less useful for clients.
                    (Err(err), _) if err.status.is_client_error() => Err(err),
                    (_, Err(err)) if err.status.is_client_error() => Err(err),
                    (err, _) => err,
                };
            }
            result
        } else {
            unreachable!("At least one authentication extension should be present.")
        };
        drop(futures);

        (
            Arc::into_inner(headers).expect("Guest should not keep the headers."),
            result,
        )
    }

    async fn public_metadata_endpoints(&self) -> Result<Vec<PublicMetadataEndpoint>, String> {
        let endpoints = self
            .authentication
            .iter()
            .map(|pool| async {
                let mut instance = pool.get().await.map_err(|err| {
                    tracing::error!("Failed to get public metadata instance: {err}");
                    "Failed to get public metadata instance".to_string()
                })?;

                wasmsafe!(instance.public_metadata().await)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(endpoints.into_iter().flatten().collect())
    }
}

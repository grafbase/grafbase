use std::sync::Arc;

use engine::ErrorResponse;
use extension_catalog::{ExtensionId, Id};
use futures::{StreamExt as _, stream::FuturesUnordered};
use runtime::extension::{AuthenticationExtension, ExtensionRequestContext, PublicMetadataEndpoint, Token};

use crate::gateway::{
    DispatchRule, ExtensionsBuilder, GatewayTestExtensions, TestExtensions, TestManifest,
    runtime::extension::builder::AnyExtension,
};

impl AuthenticationExtension for GatewayTestExtensions {
    async fn authenticate(
        &self,
        ctx: &ExtensionRequestContext,
        headers: http::HeaderMap,
        ids: &[ExtensionId],
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let mut wasm_extensions = Vec::new();
        let mut test_extensions = Vec::new();

        for id in ids {
            match self.dispatch[id] {
                DispatchRule::Wasm => wasm_extensions.push(*id),
                DispatchRule::Test => test_extensions.push(*id),
            }
        }

        let (headers, wasm_error) = if wasm_extensions.is_empty() {
            (headers, None)
        } else {
            let (headers, result) = self.wasm.authenticate(ctx, headers, &wasm_extensions).await;
            match result {
                Ok(token) => return (headers, Ok(token)),
                Err(err) => (headers, Some(err)),
            }
        };

        let (headers, test_error) = if test_extensions.is_empty() {
            (headers, None)
        } else {
            let (headers, result) = self.test.authenticate(headers, &test_extensions).await;
            match result {
                Ok(token) => return (headers, Ok(token)),
                Err(err) => (headers, Some(err)),
            }
        };

        let err = wasm_error.or(test_error).expect("Missing auth extensions");
        (headers, Err(err))
    }

    async fn public_metadata_endpoints(&self) -> Result<Vec<PublicMetadataEndpoint>, String> {
        let mut endpoints = Vec::new();
        let mut wasm_public_metadata = self.wasm.public_metadata_endpoints().await?;
        let mut native_public_metadata = self.test.public_metadata_endpoints().await?;

        endpoints.append(&mut wasm_public_metadata);
        endpoints.append(&mut native_public_metadata);

        Ok(endpoints)
    }
}

impl TestExtensions {
    async fn authenticate(
        &self,
        headers: http::HeaderMap,
        ids: &[ExtensionId],
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let guard = self.state.lock().await;
        let mut futures = guard
            .authentication
            .iter()
            .filter(|(id, _)| ids.contains(id))
            .map(|(_, instance)| instance.authenticate(&headers))
            .collect::<FuturesUnordered<_>>();

        let mut last_error = None;
        while let Some(result) = futures.by_ref().next().await {
            match result {
                Ok(token) => {
                    drop(futures);
                    return (headers, Ok(token));
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        drop(futures);

        (
            headers,
            Err(last_error.expect("At least one authentication extension should be present.")),
        )
    }

    async fn public_metadata_endpoints(&self) -> Result<Vec<PublicMetadataEndpoint>, String> {
        let guard = self.state.lock().await;
        let mut futures = guard
            .authentication
            .iter()
            .map(|(_, instance)| instance.public_metadata_endpoints())
            .collect::<FuturesUnordered<_>>();

        let mut endpoints = Vec::new();

        while let Some(result) = futures.by_ref().next().await {
            endpoints.extend(result.into_iter());
        }

        drop(futures);

        Ok(endpoints)
    }
}

pub struct AuthenticationExt {
    instance: Arc<dyn AuthenticationTestExtension>,
    name: &'static str,
    sdl: Option<&'static str>,
}

impl AuthenticationExt {
    pub fn new<T: AuthenticationTestExtension>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            name: "authentication",
            sdl: None,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_sdl(mut self, sdl: &'static str) -> Self {
        self.sdl = Some(sdl);
        self
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }
}

impl AnyExtension for AuthenticationExt {
    fn register(self, state: &mut ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: self.name.to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Authentication(Default::default()),
            sdl: None,
        });
        state.test.authentication.push((id, self.instance));
    }
}

#[async_trait::async_trait]
pub trait AuthenticationTestExtension: Send + Sync + 'static {
    async fn authenticate(&self, headers: &http::HeaderMap) -> Result<Token, ErrorResponse>;
    async fn public_metadata_endpoints(&self) -> Vec<PublicMetadataEndpoint>;
}

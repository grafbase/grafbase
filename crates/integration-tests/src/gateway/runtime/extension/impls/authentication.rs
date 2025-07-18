use std::sync::Arc;

use engine::ErrorResponse;
use extension_catalog::{ExtensionId, Id};
use futures::{StreamExt as _, stream::FuturesUnordered};
use runtime::{
    authentication::PublicMetadataEndpoint,
    extension::{AuthenticationExtension, Token},
};

use crate::gateway::{
    DispatchRule, ExtContext, ExtensionsBuilder, GatewayTestExtensions, TestExtensions, TestManifest,
    runtime::extension::builder::AnyExtension,
};

impl AuthenticationExtension<ExtContext> for GatewayTestExtensions {
    async fn authenticate(
        &self,
        ctx: &ExtContext,
        gateway_headers: http::HeaderMap,
        ids: Option<&[ExtensionId]>,
    ) -> (http::HeaderMap, Option<Result<Token, ErrorResponse>>) {
        let (wasm_extensions, test_extensions) = if let Some(ids) = ids {
            let mut wasm_extensions = Vec::new();
            let mut test_extensions = Vec::new();

            for id in ids {
                match self.dispatch[id] {
                    DispatchRule::Wasm => wasm_extensions.push(*id),
                    DispatchRule::Test => test_extensions.push(*id),
                }
            }
            (Some(wasm_extensions), Some(test_extensions))
        } else {
            (None, None)
        };

        let (headers, result) = self
            .wasm
            .authenticate(&ctx.wasm, gateway_headers, wasm_extensions.as_deref())
            .await;
        let error = match result {
            None => None,
            Some(Ok(token)) => return (headers, Some(Ok(token))),
            Some(Err(err)) => Some(err),
        };
        let (headers, result) = self.test.authenticate(ctx, headers, test_extensions.as_deref()).await;
        match (result, error) {
            (Some(Ok(token)), _) => (headers, Some(Ok(token))),
            (None, None) => (headers, None),
            (None, Some(err)) | (Some(Err(_)), Some(err)) => (headers, Some(Err(err))),
            (Some(Err(err)), _) => (headers, Some(Err(err))),
        }
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

impl AuthenticationExtension<ExtContext> for TestExtensions {
    async fn authenticate(
        &self,
        _ctx: &ExtContext,
        headers: http::HeaderMap,
        ids: Option<&[ExtensionId]>,
    ) -> (http::HeaderMap, Option<Result<Token, ErrorResponse>>) {
        let guard = self.state.lock().await;
        let mut futures = guard
            .authentication
            .iter()
            .filter(|(id, _)| ids.is_none_or(|ids| ids.contains(id)))
            .map(|(_, instance)| instance.authenticate(&headers))
            .collect::<FuturesUnordered<_>>();

        let mut last_error = None;
        while let Some(result) = futures.by_ref().next().await {
            match result {
                Ok(token) => {
                    drop(futures);
                    return (headers, Some(Ok(token)));
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        drop(futures);

        match last_error {
            None => (headers, None),
            Some(err) => (headers, Some(Err(err))),
        }
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

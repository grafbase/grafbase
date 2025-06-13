use std::sync::Arc;

use engine::ErrorResponse;
use extension_catalog::{ExtensionId, Id};
use futures::{StreamExt as _, stream::FuturesUnordered};
use runtime::extension::{AuthenticationExtension, Token};

use crate::gateway::{
    DispatchRule, ExtContext, ExtensionsBuilder, ExtensionsDispatcher, TestExtensions, TestManifest,
    runtime::extension::builder::AnyExtension,
};

impl AuthenticationExtension<ExtContext> for ExtensionsDispatcher {
    async fn authenticate(
        &self,
        ctx: &ExtContext,
        extension_ids: &[ExtensionId],
        gateway_headers: http::HeaderMap,
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let mut wasm_extensions = Vec::new();
        let mut test_extensions = Vec::new();
        for id in extension_ids {
            match self.dispatch[id] {
                DispatchRule::Wasm => wasm_extensions.push(*id),
                DispatchRule::Test => test_extensions.push(*id),
            }
        }

        assert!(
            wasm_extensions.is_empty() ^ test_extensions.is_empty(),
            "Cannot mix test & wasm authentication extensions, feel free to implement it if you need it. Shouldn't be that hard."
        );

        if !wasm_extensions.is_empty() {
            self.wasm
                .authenticate(&ctx.wasm, &wasm_extensions, gateway_headers)
                .await
        } else {
            self.test.authenticate(ctx, &test_extensions, gateway_headers).await
        }
    }
}

impl AuthenticationExtension<ExtContext> for TestExtensions {
    async fn authenticate(
        &self,
        _ctx: &ExtContext,
        extension_ids: &[ExtensionId],
        headers: http::HeaderMap,
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let mut futures = extension_ids
            .iter()
            .map(|id| async {
                let instance = self.state.lock().await.get_authentication_ext(*id);
                instance.authenticate(&headers).await
            })
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

        (headers, Err(last_error.unwrap()))
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
        state.test.authentication.insert(id, self.instance);
    }
}

#[async_trait::async_trait]
pub trait AuthenticationTestExtension: Send + Sync + 'static {
    async fn authenticate(&self, headers: &http::HeaderMap) -> Result<Token, ErrorResponse>;
}

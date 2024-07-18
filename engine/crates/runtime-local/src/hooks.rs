mod authorized;
mod pool;

use std::{collections::HashMap, sync::Arc};

use deadpool::managed::Pool;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::{AuthorizedHooks, HeaderMap, Hooks},
};
use tracing::instrument;
pub use wasi_component_loader::{ComponentLoader, Config as HooksConfig};

use self::pool::{AuthorizationHookManager, GatewayHookManager};

pub struct HooksWasi(Option<HooksWasiInner>);
type Context = Arc<HashMap<String, String>>;

struct HooksWasiInner {
    gateway_hooks: Pool<GatewayHookManager>,
    authorization_hooks: Pool<AuthorizationHookManager>,
}

impl HooksWasi {
    pub fn new(loader: Option<ComponentLoader>) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => {
                let gateway_mgr = GatewayHookManager::new(loader.clone());
                let authorization_mgr = AuthorizationHookManager::new(loader);

                let gateway_hooks = Pool::builder(gateway_mgr)
                    .build()
                    .expect("only fails if not in a runtime");

                let authorization_hooks = Pool::builder(authorization_mgr)
                    .build()
                    .expect("only fails if not in a runtime");

                Self(Some(HooksWasiInner {
                    gateway_hooks,
                    authorization_hooks,
                }))
            }
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = Context;

    #[instrument(skip_all)]
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok((Arc::new(HashMap::new()), headers));
        };

        let mut hook = inner.gateway_hooks.get().await.expect("no io, should not fail");

        hook.call(HashMap::new(), headers)
            .await
            .map(|(ctx, headers)| (Arc::new(ctx), headers))
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::Guest(err) => {
                    error_response_to_user_error(err, PartialErrorCode::BadRequest)
                }
            })
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }
}

fn error_response_to_user_error(
    error: wasi_component_loader::GuestError,
    code: PartialErrorCode,
) -> PartialGraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key.into(), value)
        })
        .collect();

    PartialGraphqlError {
        message: error.message.into(),
        code,
        extensions,
    }
}

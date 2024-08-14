mod authorized;
mod pool;
mod subgraph;

use std::{collections::HashMap, sync::Arc};

use pool::Pool;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    hooks::{AuthorizedHooks, HeaderMap, Hooks, SubgraphHooks},
};
use tracing::instrument;
use wasi_component_loader::{AuthorizationComponentInstance, GatewayComponentInstance, SubgraphComponentInstance};
pub use wasi_component_loader::{ComponentLoader, Config as HooksWasiConfig};

pub struct HooksWasi(Option<HooksWasiInner>);
type Context = Arc<HashMap<String, String>>;

struct HooksWasiInner {
    gateway: Pool<GatewayComponentInstance>,
    authorization: Pool<AuthorizationComponentInstance>,
    subgraph: Pool<SubgraphComponentInstance>,
}

impl HooksWasi {
    pub fn new(loader: Option<ComponentLoader>) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => Self(Some(HooksWasiInner {
                gateway: Pool::new(&loader),
                authorization: Pool::new(&loader),
                subgraph: Pool::new(&loader),
            })),
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = Context;

    #[instrument(skip_all)]
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), ErrorResponse> {
        let Some(ref inner) = self.0 else {
            return Ok((Arc::new(HashMap::new()), headers));
        };

        let mut hook = inner.gateway.get().await;

        hook.on_gateway_request(HashMap::new(), headers)
            .await
            .map(|(ctx, headers)| (Arc::new(ctx), headers))
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    PartialGraphqlError::internal_hook_error().into()
                }
                wasi_component_loader::Error::Guest(err) => {
                    guest_error_as_gql(err, PartialErrorCode::BadRequest).into()
                }
            })
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }

    fn subgraph(&self) -> &impl SubgraphHooks<Self::Context> {
        self
    }
}

fn guest_error_as_gql(error: wasi_component_loader::GuestError, code: PartialErrorCode) -> PartialGraphqlError {
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

use runtime::udf::{
    AuthorizerInvoker, CustomResolverInvoker, UdfError, UdfInvoker, UdfInvokerInner, UdfRequest, UdfResponse,
};
use serde::Serialize;

use crate::bridge::Bridge;

pub struct UdfInvokerImpl {
    bridge: Bridge,
}

impl UdfInvokerImpl {
    pub fn custom_resolver(bridge: Bridge) -> CustomResolverInvoker {
        UdfInvoker::new(Self { bridge })
    }

    pub fn authorizer(bridge: Bridge) -> AuthorizerInvoker {
        UdfInvoker::new(Self { bridge })
    }
}

#[async_trait::async_trait]
impl<Payload: Serialize + Send> UdfInvokerInner<Payload> for UdfInvokerImpl {
    async fn invoke(&self, ray_id: &str, request: UdfRequest<'_, Payload>) -> Result<UdfResponse, UdfError>
    where
        Payload: 'async_trait,
    {
        self.bridge.request("invoke-udf", request).await.map_err(|error| {
            log::error!(ray_id, "Resolver invocation failed with: {}", error);
            UdfError::InvocationError
        })
    }
}

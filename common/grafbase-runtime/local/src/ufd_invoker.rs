use crate::bridge::Bridge;
use grafbase_runtime::{
    self,
    udf::{CustomResolverError, CustomResolverResponse, UdfInvoker, UdfRequest},
};
use serde::Serialize;

pub struct UdfInvokerImpl {
    bridge: Bridge,
}

impl UdfInvokerImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn create_engine(bridge: Bridge) -> grafbase_runtime::udf::CustomResolversEngine {
        grafbase_runtime::udf::CustomResolversEngine::new(Box::new(Self::new(bridge)))
    }

    pub fn new(bridge: Bridge) -> Self {
        Self { bridge }
    }
}

#[async_trait::async_trait(?Send)]
impl<Payload: Serialize> UdfInvoker<Payload> for UdfInvokerImpl {
    async fn invoke(
        &self,
        ray_id: &str,
        request: UdfRequest<Payload>,
    ) -> Result<CustomResolverResponse, CustomResolverError>
    where
        Payload: 'async_trait,
    {
        self.bridge.request("invoke-udf", request).await.map_err(|error| {
            log::error!(ray_id, "Resolver invocation failed with: {}", error);
            CustomResolverError::ServerError
        })
    }
}

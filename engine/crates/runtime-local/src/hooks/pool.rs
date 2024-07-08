use std::sync::Arc;

use deadpool::managed;
use wasi_component_loader::{AuthorizationHookInstance, ComponentLoader, GatewayHookInstance};

pub(super) struct GatewayHookManager {
    component_loader: Arc<ComponentLoader>,
}

impl GatewayHookManager {
    pub fn new(component_loader: Arc<ComponentLoader>) -> Self {
        Self { component_loader }
    }
}

impl managed::Manager for GatewayHookManager {
    type Type = GatewayHookInstance;
    type Error = wasi_component_loader::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        GatewayHookInstance::new(&self.component_loader).await
    }

    async fn recycle(&self, instance: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        instance.cleanup()?;

        Ok(())
    }
}

pub(super) struct AuthorizationHookManager {
    component_loader: Arc<ComponentLoader>,
}

impl AuthorizationHookManager {
    pub fn new(component_loader: Arc<ComponentLoader>) -> Self {
        Self { component_loader }
    }
}

impl managed::Manager for AuthorizationHookManager {
    type Type = AuthorizationHookInstance;
    type Error = wasi_component_loader::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        AuthorizationHookInstance::new(&self.component_loader).await
    }

    async fn recycle(&self, instance: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        instance.cleanup()?;

        Ok(())
    }
}

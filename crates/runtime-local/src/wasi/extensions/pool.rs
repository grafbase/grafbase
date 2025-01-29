use std::ops::{Deref, DerefMut};

use deadpool::managed::{self, Manager};
use tracing::{info_span, Instrument};
use wasi_component_loader::{ChannelLogSender, ComponentLoader, Directive, ExtensionType, ExtensionsComponentInstance};

pub(super) struct Pool {
    inner: managed::Pool<ComponentManager>,
}

pub(super) struct ComponentGuard {
    inner: managed::Object<ComponentManager>,
}

impl Deref for ComponentGuard {
    type Target = ExtensionsComponentInstance;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ComponentGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Pool {
    pub(super) fn new(
        loader: ComponentLoader,
        config: ComponentManagerConfig,
        size: Option<usize>,
        access_log: ChannelLogSender,
    ) -> Self {
        let mgr = ComponentManager::new(loader, access_log, config);
        let mut builder = managed::Pool::builder(mgr);

        if let Some(size) = size {
            builder = builder.max_size(size);
        }

        let inner = builder.build().expect("only fails if not in a runtime");

        Pool { inner }
    }

    pub(super) async fn get(&self) -> ComponentGuard {
        let span = info_span!("get extension from pool");
        let inner = self.inner.get().instrument(span).await.expect("no io, should not fail");

        ComponentGuard { inner }
    }
}

pub(super) struct ComponentManagerConfig {
    pub extension_type: ExtensionType,
    pub schema_directives: Vec<Directive>,
}

pub(super) struct ComponentManager {
    component_loader: ComponentLoader,
    access_log: ChannelLogSender,
    extension_type: ExtensionType,
    schema_directives: Vec<Directive>,
}

impl ComponentManager {
    pub(super) fn new(
        component_loader: ComponentLoader,
        access_log: ChannelLogSender,
        config: ComponentManagerConfig,
    ) -> Self {
        Self {
            component_loader,
            access_log,
            extension_type: config.extension_type,
            schema_directives: config.schema_directives,
        }
    }
}

impl Manager for ComponentManager {
    type Type = ExtensionsComponentInstance;
    type Error = wasi_component_loader::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        ExtensionsComponentInstance::new(
            &self.component_loader,
            self.extension_type,
            self.schema_directives.clone(),
            self.access_log.clone(),
        )
        .await
    }

    async fn recycle(
        &self,
        instance: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if let Err(e) = instance.recycle() {
            return Err(e.into());
        }

        Ok(())
    }
}

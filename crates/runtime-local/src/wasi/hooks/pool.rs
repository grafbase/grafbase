use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use deadpool::managed;
use grafbase_telemetry::otel::opentelemetry::metrics::UpDownCounter;
use tracing::{Instrument, info_span};
use wasi_component_loader::{AccessLogSender, ComponentLoader, HooksComponentInstance};

pub(super) struct Pool {
    inner: managed::Pool<ComponentManager>,
    pool_busy_counter: UpDownCounter<i64>,
}

pub(super) struct ComponentGuard {
    inner: managed::Object<ComponentManager>,
    pool_busy_counter: UpDownCounter<i64>,
}

impl Deref for ComponentGuard {
    type Target = HooksComponentInstance;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ComponentGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for ComponentGuard {
    fn drop(&mut self) {
        self.pool_busy_counter.add(-1, &[]);
    }
}

impl Pool {
    pub(super) fn new(loader: &Arc<ComponentLoader>, size: Option<usize>, access_log: AccessLogSender) -> Self {
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
        let pool_busy_counter = meter.i64_up_down_counter("grafbase.hook.pool.instances.busy").build();

        let mgr = ComponentManager::new(loader.clone(), access_log);
        let mut builder = managed::Pool::builder(mgr);

        if let Some(size) = size {
            builder = builder.max_size(size);
        }

        let inner = builder.build().expect("only fails if not in a runtime");

        Pool {
            inner,
            pool_busy_counter,
        }
    }

    pub(super) async fn get(&self) -> ComponentGuard {
        self.pool_busy_counter.add(1, &[]);
        let span = info_span!("get hook from pool");
        let inner = self.inner.get().instrument(span).await.expect("no io, should not fail");

        ComponentGuard {
            inner,
            pool_busy_counter: self.pool_busy_counter.clone(),
        }
    }
}

pub(super) struct ComponentManager {
    component_loader: Arc<ComponentLoader>,
    pool_allocated_instances: UpDownCounter<i64>,
    access_log: AccessLogSender,
}

impl ComponentManager {
    pub(super) fn new(component_loader: Arc<ComponentLoader>, access_log: AccessLogSender) -> Self {
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
        let pool_allocated_instances = meter.i64_up_down_counter("grafbase.hook.pool.instances.size").build();

        Self {
            component_loader,
            pool_allocated_instances,
            access_log,
        }
    }
}

impl managed::Manager for ComponentManager {
    type Type = HooksComponentInstance;
    type Error = wasi_component_loader::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.pool_allocated_instances.add(1, &[]);
        HooksComponentInstance::new(&self.component_loader, self.access_log.clone()).await
    }

    async fn recycle(&self, instance: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        if let Err(e) = instance.recycle() {
            self.pool_allocated_instances.add(-1, &[]);
            return Err(e.into());
        }

        Ok(())
    }
}

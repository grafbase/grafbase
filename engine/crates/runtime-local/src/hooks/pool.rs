use std::sync::Arc;

use deadpool::managed;
use grafbase_telemetry::otel::opentelemetry::{metrics::UpDownCounter, KeyValue};
use tracing::{info_span, Instrument};
use wasi_component_loader::{ComponentLoader, RecycleableComponentInstance};

pub(super) struct Pool<T: RecycleableComponentInstance>(managed::Pool<ComponentMananger<T>>);

impl<T: RecycleableComponentInstance> Pool<T> {
    pub(super) fn new(loader: &Arc<ComponentLoader>) -> Option<Self> {
        if loader.implements_interface(T::interface_name()) {
            let mgr = ComponentMananger::<T>::new(loader.clone());

            let pool = managed::Pool::builder(mgr)
                .build()
                .expect("only fails if not in a runtime");

            Some(Pool(pool))
        } else {
            None
        }
    }

    pub(super) async fn get(&self) -> managed::Object<ComponentMananger<T>> {
        let span = info_span!("get instance from pool");
        self.0.get().instrument(span).await.expect("no io, should not fail")
    }
}

pub(super) struct ComponentMananger<T> {
    component_loader: Arc<ComponentLoader>,
    pool_size_counter: UpDownCounter<i64>,
    counter_attributes: Vec<KeyValue>,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T: RecycleableComponentInstance> ComponentMananger<T> {
    pub(super) fn new(component_loader: Arc<ComponentLoader>) -> Self {
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
        let pool_size_counter = meter.i64_up_down_counter("grafbase.hook.pool.size").init();
        let counter_attributes = vec![KeyValue::new("grafbase.hook.interface", T::interface_name())];

        Self {
            component_loader,
            pool_size_counter,
            counter_attributes,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: RecycleableComponentInstance> managed::Manager for ComponentMananger<T> {
    type Type = T;
    type Error = wasi_component_loader::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.pool_size_counter.add(1, &self.counter_attributes);
        T::new(&self.component_loader).await
    }

    async fn recycle(&self, instance: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        if let Err(e) = instance.recycle() {
            self.pool_size_counter.add(-1, &self.counter_attributes);
            return Err(e.into());
        }

        Ok(())
    }
}

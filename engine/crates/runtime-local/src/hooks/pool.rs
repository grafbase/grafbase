use std::sync::Arc;

use deadpool::managed;
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
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T: RecycleableComponentInstance> ComponentMananger<T> {
    pub(super) fn new(component_loader: Arc<ComponentLoader>) -> Self {
        Self {
            component_loader,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: RecycleableComponentInstance> managed::Manager for ComponentMananger<T> {
    type Type = T;
    type Error = wasi_component_loader::Error;
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        T::new(&self.component_loader).await
    }
    async fn recycle(&self, instance: &mut Self::Type, _: &managed::Metrics) -> managed::RecycleResult<Self::Error> {
        instance.recycle()?;
        Ok(())
    }
}

use std::sync::Arc;

use deadpool::managed;
use wasi_component_loader::{ComponentLoader, RecycleableComponentInstance};

pub(super) struct Pool<T: RecycleableComponentInstance>(managed::Pool<ComponentMananger<T>>);

impl<T: RecycleableComponentInstance> Pool<T> {
    pub(super) fn new(loader: &Arc<ComponentLoader>) -> Self {
        let mgr = ComponentMananger::<T>::new(loader.clone());
        Self(
            managed::Pool::builder(mgr)
                .build()
                .expect("only fails if not in a runtime"),
        )
    }

    pub(super) async fn get(&self) -> managed::Object<ComponentMananger<T>> {
        self.0.get().await.expect("no io, should not fail")
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

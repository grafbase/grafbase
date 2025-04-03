use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default, Clone)]
pub struct ExtContext {
    pub wasm: wasi_component_loader::SharedContext,
    pub test: DynHookContext,
}
#[derive(Default, Clone)]
pub struct DynHookContext {
    by_type: HashMap<TypeId, Arc<dyn Any + Sync + Send>>,
    by_name: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl DynHookContext {
    pub fn typed_get<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.by_type
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn typed_insert<T>(&mut self, value: T)
    where
        T: 'static + Send + Sync,
    {
        self.by_type.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get(&self, name: &str) -> Option<serde_json::Value> {
        self.by_name.lock().unwrap().get(name).cloned()
    }

    pub fn insert(&self, name: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.by_name.lock().unwrap().insert(name.into(), value.into());
    }
}

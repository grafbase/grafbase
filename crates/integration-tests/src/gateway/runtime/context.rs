use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use event_queue::EventQueue;
use runtime::extension::ExtensionContext;

#[derive(Default, Clone)]
pub struct ExtContext {
    pub wasm: wasi_component_loader::SharedContext,
    pub kv: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl ExtensionContext for ExtContext {
    fn event_queue(&self) -> &EventQueue {
        self.wasm.event_queue()
    }
}

impl ExtContext {
    pub fn get(&self, name: &str) -> Option<serde_json::Value> {
        self.kv.lock().unwrap().get(name).cloned()
    }

    pub fn insert(&self, name: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.kv.lock().unwrap().insert(name.into(), value.into());
    }
}

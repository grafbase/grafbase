use std::sync::Arc;

use event_queue::EventQueue;
use runtime::extension::ExtensionContext;

#[derive(Clone)]
pub struct WasmContext(Arc<WasmContextInner>);

impl std::ops::Deref for WasmContext {
    type Target = WasmContextInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WasmContextInner {
    pub event_queue: EventQueue,
}

impl ExtensionContext for WasmContext {
    fn event_queue(&self) -> &EventQueue {
        &self.event_queue
    }
}

impl Default for WasmContext {
    fn default() -> Self {
        WasmContext(Arc::new(WasmContextInner {
            event_queue: EventQueue::default(),
        }))
    }
}

impl WasmContext {
    pub fn new(event_queue: EventQueue) -> Self {
        WasmContext(Arc::new(WasmContextInner { event_queue }))
    }
}

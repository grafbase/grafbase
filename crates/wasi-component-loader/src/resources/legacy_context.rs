use std::sync::Arc;

use engine::{EngineOperationContext, EngineRequestContext};
use event_queue::EventQueue;

#[derive(Default, Clone)]
pub struct LegacyWasmContext(Arc<EventQueue>);

impl From<Arc<EventQueue>> for LegacyWasmContext {
    fn from(event_queue: Arc<EventQueue>) -> Self {
        Self(event_queue)
    }
}

impl From<&EngineOperationContext> for LegacyWasmContext {
    fn from(ctx: &EngineOperationContext) -> Self {
        Self(ctx.event_queue().clone())
    }
}

impl From<&EngineRequestContext> for LegacyWasmContext {
    fn from(ctx: &EngineRequestContext) -> Self {
        Self(ctx.event_queue().clone())
    }
}

impl std::ops::Deref for LegacyWasmContext {
    type Target = EventQueue;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

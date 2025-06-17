use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use event_queue::EventQueue;
use extension_catalog::ExtensionId;
use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use runtime::extension::ExtensionContext;

/// The internal per-request context storage. Accessible from all hooks throughout a single request
pub type ContextMap = HashMap<String, String>;

/// The internal per-request context storage, read-only.
#[derive(Clone)]
pub struct SharedContext(Arc<SharedContextInner>);

impl std::ops::Deref for SharedContext {
    type Target = SharedContextInner;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct SharedContextInner {
    pub(crate) authorization_states: OnceLock<Vec<(ExtensionId, Vec<u8>)>>,
    // FIXME: legacy kv for sdk 0.9
    pub(crate) kv: Arc<HashMap<String, String>>,
    /// A log channel for access logs.
    pub(crate) trace_id: TraceId,
    pub(crate) event_queue: EventQueue,
}

impl ExtensionContext for SharedContext {
    fn event_queue(&self) -> &EventQueue {
        &self.event_queue
    }
}

impl Default for SharedContext {
    fn default() -> Self {
        Self(Arc::new(SharedContextInner {
            kv: Default::default(),
            authorization_states: Default::default(),
            trace_id: TraceId::INVALID,
            event_queue: Default::default(),
        }))
    }
}

impl SharedContext {
    /// Creates a new shared context.
    pub fn new(event_queue: EventQueue) -> Self {
        Self(Arc::new(SharedContextInner {
            event_queue,
            kv: Default::default(),
            authorization_states: Default::default(),
            trace_id: TraceId::INVALID,
        }))
    }
}

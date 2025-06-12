use std::{collections::HashMap, sync::Arc};

use extension_catalog::ExtensionId;
use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use runtime::extension::ExtensionContext;

use crate::resources::EventQueue;

/// The internal per-request context storage. Accessible from all hooks throughout a single request
pub type ContextMap = HashMap<String, String>;

type AuthorizationState = Arc<tokio::sync::RwLock<Vec<(ExtensionId, Vec<u8>)>>>;

/// The internal per-request context storage, read-only.
#[derive(Clone)]
pub struct SharedContext {
    /// Key-value storage.
    pub(crate) kv: Arc<HashMap<String, String>>,
    pub(crate) authorization_state: AuthorizationState,
    /// A log channel for access logs.
    pub(crate) trace_id: TraceId,
    pub(crate) event_queue: EventQueue,
}

impl ExtensionContext for SharedContext {
    type EventQueue = EventQueue;

    fn event_queue(&self) -> &Self::EventQueue {
        &self.event_queue
    }
}

// FIXME: Remove me once hooks & extensions context are merged.
impl Default for SharedContext {
    fn default() -> Self {
        Self {
            kv: Default::default(),
            authorization_state: Default::default(),
            trace_id: TraceId::INVALID,
            event_queue: Default::default(),
        }
    }
}

impl SharedContext {
    /// Creates a new shared context.
    pub fn new(kv: Arc<HashMap<String, String>>, trace_id: TraceId, event_queue: EventQueue) -> Self {
        Self {
            kv,
            trace_id,
            event_queue,
            ..Default::default()
        }
    }
}

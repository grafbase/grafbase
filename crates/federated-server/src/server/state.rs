use std::sync::Arc;

use super::gateway::EngineWatcher;

pub(super) struct ServerStateInner<SR> {
    /// The gateway responsible for handling engine communication.
    pub gateway: EngineWatcher,

    /// The maximum size in bytes for the request body.
    pub request_body_limit_bytes: usize,

    /// The server runtime, defining how to trigger IO depending on the platform.
    #[cfg_attr(not(feature = "lambda"), allow(unused))]
    pub server_runtime: SR,
}

pub(super) struct ServerState<SR> {
    inner: Arc<ServerStateInner<SR>>,
}

impl<SR> Clone for ServerState<SR> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<SR> ServerState<SR> {
    pub(super) fn new(gateway: EngineWatcher, request_body_limit_bytes: usize, server_runtime: SR) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                gateway,
                server_runtime,
                request_body_limit_bytes,
            }),
        }
    }
}

impl<SR> std::ops::Deref for ServerState<SR> {
    type Target = ServerStateInner<SR>;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

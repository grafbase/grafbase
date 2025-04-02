use super::gateway::EngineWatcher;
use std::sync::Arc;

pub struct ServerStateInner<R: engine::Runtime, SR> {
    /// The gateway responsible for handling engine communication.
    pub engine: EngineWatcher<R>,

    /// The maximum size in bytes for the request body.
    pub request_body_limit_bytes: usize,

    /// The server runtime, defining how to trigger IO depending on the platform.
    #[cfg_attr(not(feature = "lambda"), allow(unused))]
    pub server_runtime: SR,
}

pub struct ServerState<R: engine::Runtime, SR> {
    inner: Arc<ServerStateInner<R, SR>>,
}

impl<R: engine::Runtime, SR> Clone for ServerState<R, SR> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R: engine::Runtime, SR> ServerState<R, SR> {
    pub(super) fn new(engine: EngineWatcher<R>, request_body_limit_bytes: usize, server_runtime: SR) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                engine,
                server_runtime,
                request_body_limit_bytes,
            }),
        }
    }
}

impl<R: engine::Runtime, SR> std::ops::Deref for ServerState<R, SR> {
    type Target = ServerStateInner<R, SR>;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

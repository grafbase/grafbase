use runtime::async_runtime::{AsyncRuntime, AsyncRuntimeInner};

pub struct TokioCurrentRuntime;

impl TokioCurrentRuntime {
    pub fn runtime() -> AsyncRuntime {
        AsyncRuntime::new(Self)
    }
}

impl AsyncRuntimeInner for TokioCurrentRuntime {
    fn spawn(&self, future: futures_util::future::BoxFuture<'static, ()>) {
        tokio::runtime::Handle::current().spawn(future);
    }
}

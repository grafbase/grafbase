use futures_util::Future;

#[cfg(target_arch = "wasm32")]
pub fn make_send_on_wasm<T>(future: impl Future<Output = T>) -> impl Future<Output = T> + Send {
    send_wrapper::SendWrapper::new(future)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn make_send_on_wasm<F>(future: F) -> F
where
    F: Future + Send,
{
    future
}

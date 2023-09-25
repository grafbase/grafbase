use core::future::Future;

#[cfg(target_arch = "wasm32")]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(future);
}

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

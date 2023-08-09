use futures::Future;

#[cfg(target_arch = "wasm32")]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(future);
}

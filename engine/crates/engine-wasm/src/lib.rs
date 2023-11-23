use std::sync::Arc;
use wasm_bindgen::prelude::*;

struct Executor;

impl gateway_core::Executor for Executor {
    type Error = String;
    type Context = ();
    type Response = String;

    // Caching can defer the actual execution of the request when the data is stale for example.
    // To simplify our code, instead of having a 'ctx lifetime, we expect those "background"
    // futures to be 'static. Hence this method requires an Arc<Self>.
    async fn execute(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: Request,
    ) -> Result<engine::Response, Self::Error> {
        todo!()
    }

    async fn execute_stream(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
        streaming_format: crate::StreamingFormat,
    ) -> Result<Self::Response, Self::Error> {
        Err("Streaming responses are not supported".to_string())
    }
}

struct GrafbaseGateway {
    gateway: gateway_core::Gateway<(), ()>,
}

#[wasm_bindgen]
pub fn make_config(schema: &str) -> Vec<u8> {
    let executor = todo!();
    let cache = todo!();
    let cache_config = todo!();
    let authorizer = todo!();
    let gateway = gateway_core::Gateway::new(executor, cache, cache_config, authorizer);

    // let registry = parse_sdl(schema).unwrap();
    b"meowmeowmeow".to_vec()
}

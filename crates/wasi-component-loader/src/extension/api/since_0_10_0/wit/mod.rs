pub mod authorization;
pub mod headers;
pub mod resolver;
pub mod token;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_10_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/headers/headers": crate::resources::Headers,
        "grafbase:sdk/directive": crate::extension::api::since_0_9_0::wit::directive,
        "grafbase:sdk/access-log": crate::extension::api::since_0_9_0::wit::access_log,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_9_0::wit::nats_client,
        "grafbase:sdk/http-client": crate::extension::api::since_0_9_0::wit::http_client,
        "grafbase:sdk/cache": crate::extension::api::since_0_9_0::wit::cache,
        "grafbase:sdk/error": crate::extension::api::since_0_9_0::wit::error,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

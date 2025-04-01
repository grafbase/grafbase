pub mod authorization_types;
pub mod field_resolver_types;
pub mod schema;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_14_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/directive": crate::extension::api::since_0_9_0::wit::directive,
        "grafbase:sdk/access-log": crate::extension::api::since_0_9_0::wit::access_log,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_9_0::wit::nats_client,
        "grafbase:sdk/http-client": crate::extension::api::since_0_9_0::wit::http_client,
        "grafbase:sdk/cache": crate::extension::api::since_0_9_0::wit::cache,
        "grafbase:sdk/error": crate::extension::api::since_0_9_0::wit::error,
        "grafbase:sdk/token": crate::extension::api::since_0_10_0::wit::token,
        "grafbase:sdk/headers": crate::extension::api::since_0_10_0::wit::headers,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

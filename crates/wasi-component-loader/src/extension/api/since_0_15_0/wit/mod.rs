pub mod postgres;
pub mod schema;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_15_0/",
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
        "grafbase:sdk/postgres/pg-pool": crate::resources::PgPool,
        "grafbase:sdk/postgres/pg-connection": crate::resources::PgConnection,
        "grafbase:sdk/postgres/pg-transaction": crate::resources::PgTransaction,
        "grafbase:sdk/postgres/pg-row": crate::resources::PgRow,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/field-resolver-types": crate::extension::api::since_0_14_0::wit::field_resolver_types,
        "grafbase:sdk/resolver-types": crate::extension::api::since_0_14_0::wit::resolver_types,
        "grafbase:sdk/authorization-types": crate::extension::api::since_0_14_0::wit::authorization_types,
        "grafbase:sdk/selection-set-resolver-types": crate::extension::api::since_0_14_0::wit::selection_set_resolver_types,
        "grafbase:sdk/schema": schema,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

#![allow(clippy::panic)]

mod addresses;
mod cargo;
mod client;
mod command_handles;
mod gateway_builder;
mod request;
mod response;

mod batch_request;
mod batch_response;
pub mod mocks;

use std::sync::OnceLock;

use tokio::runtime::Runtime;

pub use self::{
    addresses::listen_address,
    batch_request::TestBatchRequest,
    batch_response::GraphqlHttpBatchResponse,
    cargo::cargo_bin,
    client::Client,
    command_handles::CommandHandles,
    gateway_builder::{ConfigContent, GatewayBuilder},
    request::{GraphQlRequestBody, TestRequest},
    response::GraphqlHttpResponse,
};

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

pub fn clickhouse_client() -> &'static ::clickhouse::Client {
    static CLIENT: OnceLock<::clickhouse::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        ::clickhouse::Client::default()
            .with_url("http://localhost:8124")
            .with_user("default")
            .with_database("otel")
    })
}

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[cfg(test)]
mod appease_the_unused_deps_lint {
    use async_graphql_parser as _;
    use futures_util as _;
    use grafbase_graphql_introspection as _;
    use graphql_composition as _;
    use graphql_mocks as _;
    use handlebars as _;
    use indoc as _;
    use insta as _;
    use rand as _;
    use serde_with as _;
    use ulid as _;
    use wiremock as _;
}

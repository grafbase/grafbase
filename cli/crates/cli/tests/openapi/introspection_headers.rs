use std::net::SocketAddr;

use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

use crate::utils::environment::Environment;

use super::start_grafbase;

#[tokio::test(flavor = "multi_thread")]
async fn introspection_headers_test() {
    let mock_server = wiremock::MockServer::start().await;
    mount_petstore_spec_that_expects_headers(&mock_server).await;

    let mut env = Environment::init_async().await;

    // This should fail if the header doesn't tell us its friday
    start_grafbase(&mut env, schema(mock_server.address())).await;
}

async fn mount_petstore_spec_that_expects_headers(server: &wiremock::MockServer) {
    Mock::given(method("GET"))
        .and(path("spec.json"))
        .and(header("day", "friday"))
        .respond_with(ResponseTemplate::new(200).set_body_string(include_str!("petstore.json")))
        .mount(server)
        .await;
}

fn schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://{address}/spec.json",
            introspectionHeaders: [{{ name: "day", value: "friday" }}]
          )
        "#
    )
}

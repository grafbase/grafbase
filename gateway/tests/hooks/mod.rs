use std::collections::BTreeMap;

use handlebars::Handlebars;
use serde_json::json;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{header, method},
};

use crate::{load_schema, runtime, with_static_server};

#[test]
fn extension_loads_and_passes_headers() {
    let wasi_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../crates/integration-tests/data/extensions/crates/hooks/build"
    );

    let config = indoc::formatdoc! {r#"
        [graph]
        introspection = true

        [extensions.hooks]
        path = "{wasi_path}"
        stdout = true
        stderr = true

        [[headers]]
        rule = "forward"
        name = "x-incoming-header"

        [extensions.hooks.config]
        incoming_header.key = "X-Incoming-Header"
        incoming_header.value = "kekw"
    "#};

    let server = runtime().block_on(async move {
        let server = wiremock::MockServer::start().await;

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "me": {
                    "id": "1",
                    "username": "Alice",
                }
            }
        }));

        Mock::given(method("POST"))
            .and(header("x-incoming-header", "kekw"))
            .respond_with(response)
            .mount(&server)
            .await;

        server
    });

    let mut hb = Handlebars::new();
    hb.register_template_string("t1", load_schema("small")).unwrap();

    let mut data = BTreeMap::new();
    data.insert("subgraph_endpoint", format!("http://{}", server.address()));

    let schema = hb.render("t1", &data).unwrap();

    println!("{config}");
    println!("{schema}");

    with_static_server(config, &schema, None, None, |client| async move {
        let resp = client
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": {
            "me": {
              "id": "1"
            }
          }
        }
        "###);

        server.received_requests().await;
    });
}

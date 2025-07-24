use graphql_mocks::EchoSchema;
use indoc::indoc;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn incoming_header() {
    let response = runtime().block_on(async move {
        let config = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "x-incoming-header"

            [extensions.hooks-17.config]
            incoming_header.key = "X-Incoming-Header"
            incoming_header.value = "kekw"
        "#};

        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("hooks-17")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let query = indoc! {r#"
            query {
                header(name: "x-incoming-header")
            }
        "#};

        engine.post(query).await
    });

    insta::assert_snapshot!(response, @r#"
    {
      "data": {
        "header": "kekw"
      }
    }
    "#);
}

#[test]
fn outgoing_header() {
    let response = runtime().block_on(async move {
        let config = indoc! {r#"
            [extensions.hooks-17.config]
            outgoing_header.key = "X-Outgoing-Header"
            outgoing_header.value = "kekw"
        "#};

        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("hooks-17")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let query = indoc! {r#"
            query {
                headers { name }
            }
        "#};

        engine.post(query).await
    });

    assert_eq!(
        response.headers.get("x-outgoing-header").and_then(|h| h.to_str().ok()),
        Some("kekw")
    );
}

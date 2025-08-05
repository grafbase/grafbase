use graphql_mocks::{EchoSchema, Schema, Subgraph as _};
use indoc::indoc;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn subgraph_header_change() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_extension("hooks-19")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let query = indoc! {r#"
            query {
                header(name: "x-test")
            }
        "#};

        let response = gateway.post(query).await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "header": null
          }
        }
        "#);

        let gateway = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.hooks-19.config.on_subgraph_request]
                header_name = "X-Test"
                header_value = "test-value"
            "#,
            )
            .with_extension("hooks-19")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let query = indoc! {r#"
            query {
                header(name: "x-test")
            }
        "#};

        let response = gateway.post(query).await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "header": "test-value"
          }
        }
        "#);
    });
}

#[test]
fn subgraph_url_change() {
    runtime().block_on(async move {
        let server = graphql_mocks::TeaShop::default().start().await;
        let gateway = Gateway::builder()
            .with_toml_config(format!(
                r#"
                [extensions.hooks-19.config.on_subgraph_request]
                url = "{}"
                "#,
                server.url(),
            ))
            .with_extension("hooks-19")
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                type Query {
                    recommendedTeas: [Tea]
                }
                type Tea {
                    name: String
                }
                "#,
            ))
            .build()
            .await;

        let query = indoc! {r#"
            query {
                recommendedTeas {
                    name
                }
            }
        "#};

        let response = gateway.post(query).await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "recommendedTeas": [
              {
                "name": "Earl Grey"
              },
              {
                "name": "Tuóchá"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn receives_header_from_on_request() {
    runtime().block_on(async move {
        let config = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "x-incoming-header"

            [extensions.hooks-19.config]
            incoming_header.key = "X-Incoming-Header"
            incoming_header.value = "kekw"
            on_subgraph_request.rename_header.from = "x-incoming-header"
            on_subgraph_request.rename_header.to = "x-renamed-header"
        "#};

        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_extension("hooks-19")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let query = indoc! {r#"
            query {
                header(name: "x-renamed-header")
            }
        "#};

        let response = engine.post(query).await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "header": "kekw"
          }
        }
        "#);
    })
}

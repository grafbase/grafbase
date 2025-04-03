use integration_tests::{gateway::Gateway, runtime};

#[test]
fn sdk_014() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "b",
                r#"
                extend schema @link(url: "selection-set-resolver-014-1.0.0", import: ["@init"]) @init
                scalar JSON
                type Query {
                    test: JSON
                }
                "#,
            )
            .with_extension("selection-set-resolver-014")
            .with_toml_config(
                r#"
                [extensions.selection-set-resolver-014.config]
                value = "Hi there!"
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { test }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": "Hi there!"
          }
        }
        "#);
    });
}

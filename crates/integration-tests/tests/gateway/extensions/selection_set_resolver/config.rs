use integration_tests::{gateway::Gateway, runtime};

#[test]
fn can_read_config() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-config",
                r#"
                extend schema @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"]) @init
                scalar JSON
                type Query {
                    test: JSON
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .with_toml_config(
                r#"
                [extensions.selection-set-resolver-015.config]
                value = "Hi there!"
                other = { complex = [2, {test = 3}]}
                "#,
            )
            .build()
            .await;

        let response = engine.post(r#"query { test }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "other": {
                "complex": [
                  2,
                  {
                    "test": 3
                  }
                ]
              },
              "value": "Hi there!"
            }
          }
        }
        "#);
    });
}

#[test]
fn can_fail_reading_config_with_nullable_field() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "echo-config",
                r#"
                extend schema @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"]) @init
                scalar JSON
                type Query {
                    test: JSON
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .with_toml_config(
                r#"
                [extensions.selection-set-resolver-015.config]
                error = "This is an error"
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"Error { extensions: [], message: "This is an error" }"#);
    });
}

#[test]
fn can_fail_reading_config_with_required_field() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "echo-config",
                r#"
                extend schema @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"]) @init
                scalar JSON
                type Query {
                    test: JSON!
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .with_toml_config(
                r#"
                [extensions.selection-set-resolver-015.config]
                error = "This is an error"
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"Error { extensions: [], message: "This is an error" }"#);
    });
}

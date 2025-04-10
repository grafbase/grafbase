use super::setup::*;

#[tokio::test]
async fn grafbase_dev_basic() {
    // FIXME: MacOS & Windows have troubles with those tests in the CI. MacOS works just fine though
    // locally...
    if !cfg!(target_os = "linux") {
        return;
    }

    let dev = GrafbaseDevConfig::new()
        .with_subgraph(graphql_mocks::EchoSchema)
        .start()
        .await;

    let response = dev.graphql_simple("query { int(input: 1337) }").await;

    insta::assert_json_snapshot!(response);
}

#[tokio::test]
async fn local_extension() {
    TestExtensions::Echo.build().await;
    // FIXME: MacOS & Windows have troubles with those tests in the CI. MacOS works just fine though
    // locally...
    if !cfg!(target_os = "linux") {
        return;
    }

    let extension_path = TestExtensions::Echo.build_dir_path();
    let extension_path = extension_path.display();

    let dev = GrafbaseDevConfig::new()
        .with_gateway_config(format!(
            r#"
            [extensions.echo-extension]
            version = "0.1.0"
            networking = true
            path = '{extension_path}'
            "#,
        ))
        .with_sdl_only_subgraph(
            "extension-only",
            format!(
                r#"
            extend schema @link(url: "file://{extension_path}", as: "echo")

            type Query {{
                saySomething: String @echo__hello(to: "Arnold")
            }}
            "#
            ),
        )
        .start()
        .await;

    let response = dev.graphql_simple("query { saySomething }").await;

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "saySomething": "Hello, Arnold"
      }
    }
    "#);
}

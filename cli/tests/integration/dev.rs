use super::setup::*;

#[tokio::test]
async fn grafbase_dev_basic() {
    let dev = GrafbaseDevConfig::new()
        .with_subgraph(graphql_mocks::EchoSchema)
        .start()
        .await;

    let response = dev.graphql_simple("query { int(input: 1337) }").await;

    insta::assert_json_snapshot!(response);
}

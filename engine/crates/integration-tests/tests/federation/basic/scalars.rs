use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime, MockGraphQlServer};

#[test]
fn supports_custom_scalars() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine.execute("query { favoriteRepository }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "favoriteRepository": {
          "owner": "rust-lang",
          "name": "rust"
        }
      }
    }
    "###);
}

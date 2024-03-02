use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::federation::GraphqlResponse;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider};
use integration_tests::{
    federation::EngineV2Ext,
    openid::{AUDIENCE, JWKS_URI, JWKS_URI_2},
    runtime,
};

#[test]
fn test_provider() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!(
                r#"extend schema @authz(providers: [
                    {{ name: "my-jwt", type: jwt, jwks: {{ url: "{JWKS_URI}" }} }}
                    {{ name: "my-jwt-2", type: jwt, jwks: {{ url: "{JWKS_URI_2}" }} }}
                ])"#
            ))
            .finish()
            .await;

        // this one should work with `my-jwt` provider
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);

        // this one should work with `my-jwt-2` provider
        let token = OryHydraOpenIDProvider::second_provider()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("audience", AUDIENCE)])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);

        // No token
        let response: GraphqlResponse = engine.execute("query { serverVersion }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);
    });
}

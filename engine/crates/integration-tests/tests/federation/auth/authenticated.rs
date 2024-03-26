use futures::Future;
use gateway_v2::Gateway;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationGateway},
    openid::{CoreClientExt, OryHydraOpenIDProvider, JWKS_URI},
    runtime,
};

pub(super) fn with_secure_schema<F, O>(f: impl FnOnce(TestFederationGateway) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let secure_mock = MockGraphQlServer::new(SecureSchema::default()).await;

        let engine = Gateway::builder()
            .with_schema("secure", &secure_mock)
            .await
            .with_supergraph_config(format!(
                r#"extend schema @authz(providers: [
                {{ name: "my-jwt", type: jwt, jwks: {{ url: "{JWKS_URI}" }} }},
                {{ type: anonymous }}
            ])"#
            ))
            .finish()
            .await;

        f(engine).await
    })
}

#[test]
fn authenticated() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response = engine
            .execute("query { anonymous }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "anonymous": "Hello anonymous!"
          }
        }
        "###);

        let response = engine
            .execute("query { mustBeAuthenticated }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustBeAuthenticated": "You are authenticated"
          }
        }
        "###);
    });
}

#[test]
fn not_authenticated() {
    with_secure_schema(|engine| async move {
        let response = engine.execute("query { anonymous }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "anonymous": "Hello anonymous!"
          }
        }
        "###);

        let response = engine.execute("query { mustBeAuthenticated }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not authenticated",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ]
            }
          ]
        }
        "###);
    });
}

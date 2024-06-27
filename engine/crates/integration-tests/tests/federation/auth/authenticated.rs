use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationEngine},
    openid::{CoreClientExt, OryHydraOpenIDProvider, JWKS_URI},
    runtime,
};

pub(super) fn with_secure_schema<F, O>(f: impl FnOnce(TestFederationEngine) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let secure_mock = MockGraphQlServer::new(SecureSchema::default()).await;

        let engine = Engine::builder()
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
            .execute("query { check { anonymous } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "anonymous": "Hello anonymous!"
            }
          }
        }
        "###);

        let response = engine
            .execute("query { check { mustBeAuthenticated } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustBeAuthenticated": "You are authenticated"
            }
          }
        }
        "###);
    });
}

#[test]
fn not_authenticated() {
    with_secure_schema(|engine| async move {
        let response = engine.execute("query { check { anonymous } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "anonymous": "Hello anonymous!"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { mustBeAuthenticated } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthenticated",
              "path": [
                "check",
                "mustBeAuthenticated"
              ]
            }
          ]
        }
        "###);
    });
}

#[test]
fn authenticated_on_nullable_field() {
    with_secure_schema(|engine| async move {
        let response = engine.execute("query { nullableCheck { mustBeAuthenticated } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "nullableCheck": null
          },
          "errors": [
            {
              "message": "Unauthenticated",
              "path": [
                "nullableCheck",
                "mustBeAuthenticated"
              ]
            }
          ]
        }
        "###);
    });
}

#[test]
fn authenticated_on_union() {
    with_secure_schema(|engine| async move {
        let response = engine
            .execute("query { entity { ... on Check { mustBeAuthenticated } } }")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthenticated",
              "path": [
                "entity",
                "mustBeAuthenticated"
              ]
            }
          ]
        }
        "###);
    });
}

#[test]
fn authenticated_on_list_with_nullable_items() {
    with_secure_schema(|engine| async move {
        let response = engine
            .execute(
                r###"
                query {
                    entities {
                        ... on Check { mustBeAuthenticated }
                        ... on User { name }
                    }
                }
                "###,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "entities": [
              {
                "name": "rusty"
              },
              null
            ]
          },
          "errors": [
            {
              "message": "Unauthenticated",
              "path": [
                "entities",
                1,
                "mustBeAuthenticated"
              ]
            }
          ]
        }
        "###);
    });
}

#[test]
fn authenticated_on_list_with_required_items() {
    with_secure_schema(|engine| async move {
        let response = engine
            .execute(
                r###"
                query {
                    entitiesWithoutCheck {
                        ... on Check { mustBeAuthenticated }
                        ... on User { name }
                    }
                }
                "###,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "entitiesWithoutCheck": [
              {
                "name": "rusty"
              }
            ]
          }
        }
        "###);
    });
}

use super::authenticated::with_secure_schema;
use graphql_mocks::SecureSchema;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider, READ_SCOPE, WRITE_SCOPE};

#[test]
fn anonymous_does_not_any_scope() {
    with_secure_schema(|engine| async move {
        let response = engine.post("query { check { mustHaveReadScope } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn no_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response = engine
            .post("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        // We shouldn't have requested the field.
        let requests = engine.drain_graphql_requests_sent_to::<SecureSchema>();
        insta::assert_json_snapshot!(requests, @r#"
        [
          {
            "query": "query { check { __typename } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);
    });
}

#[test]
fn has_read_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", READ_SCOPE)])
            .await;

        let response = engine
            .post("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadScope": "You have read scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveWriteScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadOrWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadOrWriteScope": "You have either read or write scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadAndWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveReadAndWriteScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn has_write_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", WRITE_SCOPE)])
            .await;

        let response = engine
            .post("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post("query { check { mustHaveWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveWriteScope": "You have write scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadOrWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadOrWriteScope": "You have either read or write scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadAndWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Insufficient scopes",
              "path": [
                "check",
                "mustHaveReadAndWriteScope"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn has_read_and_write_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", &format!("{READ_SCOPE} {WRITE_SCOPE}"))])
            .await;

        let response = engine
            .post("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadScope": "You have read scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveWriteScope": "You have write scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadOrWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadOrWriteScope": "You have either read or write scope"
            }
          }
        }
        "###);

        let response = engine
            .post("query { check { mustHaveReadAndWriteScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "mustHaveReadAndWriteScope": "You have read and write scopes"
            }
          }
        }
        "###);
    });
}

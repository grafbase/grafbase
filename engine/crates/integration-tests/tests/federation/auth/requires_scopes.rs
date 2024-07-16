use super::authenticated::with_secure_schema;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider, READ_SCOPE, WRITE_SCOPE};

#[test]
fn anonymous_does_not_any_scope() {
    with_secure_schema(|engine| async move {
        let response = engine.execute("query { check { mustHaveReadScope } }").await;
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
            .execute("query { check { mustHaveReadScope } }")
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
        insta::assert_json_snapshot!(engine.get_recorded_subrequests(), @r###"
        [
          {
            "subgraph_name": "secure",
            "request_body": {
              "query": "query {\n  check {\n    __typename\n  }\n}\n",
              "variables": {}
            },
            "response_body": {
              "data": {
                "check": {
                  "__typename": "Check"
                }
              }
            }
          }
        ]
        "###);
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
            .execute("query { check { mustHaveReadScope } }")
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
            .execute("query { check { mustHaveWriteScope } }")
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
            .execute("query { check { mustHaveReadOrWriteScope } }")
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
            .execute("query { check { mustHaveReadAndWriteScope } }")
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
            .execute("query { check { mustHaveReadScope } }")
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
            .execute("query { check { mustHaveWriteScope } }")
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
            .execute("query { check { mustHaveReadOrWriteScope } }")
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
            .execute("query { check { mustHaveReadAndWriteScope } }")
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
            .execute("query { check { mustHaveReadScope } }")
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
            .execute("query { check { mustHaveWriteScope } }")
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
            .execute("query { check { mustHaveReadOrWriteScope } }")
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
            .execute("query { check { mustHaveReadAndWriteScope } }")
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

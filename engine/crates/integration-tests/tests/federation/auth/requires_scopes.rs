use super::authenticated::with_secure_schema;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider, READ_SCOPE, WRITE_SCOPE};

#[test]
fn anonymous_does_not_any_scope() {
    with_secure_schema(|engine| async move {
        let response = engine.execute("query { mustHaveReadScope }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

#[test]
fn no_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response = engine
            .execute("query { mustHaveReadScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

#[test]
fn has_read_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", READ_SCOPE)])
            .await;

        let response = engine
            .execute("query { mustHaveReadScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadScope": "You have read scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

        let response = engine
            .execute("query { mustHaveReadOrWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadOrWriteScope": "You have either read or write scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveReadAndWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

#[test]
fn has_write_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", WRITE_SCOPE)])
            .await;

        let response = engine
            .execute("query { mustHaveReadScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

        let response = engine
            .execute("query { mustHaveWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveWriteScope": "You have write scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveReadOrWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadOrWriteScope": "You have either read or write scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveReadAndWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Not allowed",
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

#[test]
fn has_read_and_write_scope() {
    with_secure_schema(|engine| async move {
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("scope", &format!("{READ_SCOPE} {WRITE_SCOPE}"))])
            .await;

        let response = engine
            .execute("query { mustHaveReadScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadScope": "You have read scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveWriteScope": "You have write scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveReadOrWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadOrWriteScope": "You have either read or write scope"
          }
        }
        "###);

        let response = engine
            .execute("query { mustHaveReadAndWriteScope }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "mustHaveReadAndWriteScope": "You have read and write scopes"
          }
        }
        "###);
    });
}

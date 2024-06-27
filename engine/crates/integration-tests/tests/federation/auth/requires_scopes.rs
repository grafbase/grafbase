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
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
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
            .execute("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
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
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveWriteScope"
              ]
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
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveReadAndWriteScope"
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
            .execute("query { check { mustHaveReadScope } }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveReadScope"
              ]
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
              "message": "Not allowed: insufficient scopes",
              "path": [
                "check",
                "mustHaveReadAndWriteScope"
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

use expect_test::expect;
use grafbase_engine_integration_tests::{with_mongodb, with_namespaced_mongodb};
use indoc::{formatdoc, indoc};
use serde_json::json;

#[test]
fn id_not_found() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let query = indoc! {r#"
            mutation {
              userDelete(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
                deletedCount
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDelete": {
              "deletedCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn id_found() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.insert_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            mutation {{
              userDelete(by: {{ id: "{id}" }}) {{
                deletedCount
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDelete": {
              "deletedCount": 1
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_namespaced_mongodb("mongo", schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.insert_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            mutation {{
              mongo {{
                userDelete(by: {{ id: "{id}" }}) {{
                  deletedCount
                }}
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "mongo": {
              "userDelete": {
                "deletedCount": 1
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

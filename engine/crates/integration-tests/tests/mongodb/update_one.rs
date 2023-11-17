use expect_test::expect;
use indoc::{formatdoc, indoc};
use integration_tests::{with_mongodb, with_namespaced_mongodb, GetPath};
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
              userUpdate(
                by: { id: "5ca4bbc7a2dd94ee5816238d" },
                input: { name: { set: "Derp" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userUpdate": {
              "matchedCount": 0,
              "modifiedCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "mongo", collection: "users") {
          name: String!
        }
    "#};

    let response = with_namespaced_mongodb("mongo", schema, |api| async move {
        let query = indoc! {r#"
            mutation {
              mongo {
                userUpdate(
                  by: { id: "5ca4bbc7a2dd94ee5816238d" },
                  input: { name: { set: "Derp" } }
                ) {
                  matchedCount
                  modifiedCount
                }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "mongo": {
              "userUpdate": {
                "matchedCount": 0,
                "modifiedCount": 0
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn set() {
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

        let mutation = formatdoc! {r#"
            mutation {{
              userUpdate(
                by: {{ id: "{id}" }},
                input: {{ name: {{ set: "Derp" }} }}
              ) {{
                matchedCount
                modifiedCount
              }}
            }}
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdate":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name } }  
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "name": "Derp"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

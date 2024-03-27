//! Some tests for how the MongoDB connector interacts with the rest of the schema parsing

use std::future::IntoFuture;

use expect_test::expect;
use indoc::indoc;
use integration_tests::with_mongodb;
use serde_json::json;

#[test]
fn join_with_mongo_collection_types() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!

          selfList(first: Int, after: String): UserConnection @join(
            select: "userCollection(first: $first, after: $after, filter: {id: {eq: $id}})"
          )
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.insert_one("users", document).await.inserted_id;

        let query = indoc! {r#"
            query($id: ID!) {
              user(by: { id: $id }) {
                name
                selfList(first: 10) {
                  edges {
                    node {
                      name
                    }
                  }
                }
              }
            }
        "#};

        api.engine()
            .execute(query)
            .variables(json!({
                "id": id
            }))
            .into_future()
            .await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Bob",
              "selfList": {
                "edges": [
                  {
                    "node": {
                      "name": "Bob"
                    }
                  }
                ]
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn join_with_mongo_model_type() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!

          self: User @join(
            select: "user(by: {id: $id})"
          )
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.insert_one("users", document).await.inserted_id;

        let query = indoc! {r#"
            query($id: ID!) {
              user(by: { id: $id }) {
                name
                self {
                    name
                }
              }
            }
        "#};

        api.engine()
            .execute(query)
            .variables(json!({
                "id": id
            }))
            .into_future()
            .await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Bob",
              "self": {
                "name": "Bob"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

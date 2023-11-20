use expect_test::expect;
use indoc::indoc;
use integration_tests::{with_mongodb, with_namespaced_mongodb, GetPath};
use serde_json::json;

#[test]
fn nothing_found() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let query = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { name: { set: "Alice" } }
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
            "userUpdateMany": {
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
                userUpdateMany(
                  filter: { name: { eq: "Bob" } },
                  input: { name: { set: "Alice" } }
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
              "userUpdateMany": {
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
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
            { "age": 40, "name": "Rachel" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { set: 40 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 40
                  }
                },
                {
                  "node": {
                    "name": "Rachel",
                    "age": 40
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn combining_operators() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { set: 40 }, name: { set: "Janice" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Janice",
                    "age": 40
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn unset_false() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
            { "age": 40, "name": "Rachel" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { unset: false } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        api.execute(mutation).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userUpdateMany": {
              "matchedCount": 0,
              "modifiedCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn unset_true() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
            { "age": 40, "name": "Rachel" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { unset: true } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": null
                  }
                },
                {
                  "node": {
                    "name": "Rachel",
                    "age": 40
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn current_timestamp_false() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
          time: Timestamp
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "time": 1_565_545_684, "name": "Alice" },
            { "time": 1_565_545_684, "name": "Bob" },
            { "time": 1_565_545_684, "name": "Rachel" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { time: { currentDate: false } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        api.execute(mutation).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userUpdateMany": {
              "matchedCount": 0,
              "modifiedCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn current_timestamp_true() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
          time: Timestamp
        }
    "#};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Node {
        name: String,
        time: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Edge {
        node: Node,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UserCollection {
        edges: Vec<Edge>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        user_collection: UserCollection,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Result {
        data: Data,
    }

    with_mongodb(schema, |api| async move {
        let documents = json!([
            { "time": 1_565_545_684, "name": "Alice" },
            { "time": 1_565_545_684, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { time: { currentDate: true } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name time } }  
              }
            }
        "};

        let result = api.execute(query).await;
        let mut deserialized: Result = serde_json::from_str(&result.as_json_string()).unwrap();

        let bob = deserialized.data.user_collection.edges.pop().unwrap().node;
        let alice = deserialized.data.user_collection.edges.pop().unwrap().node;

        assert_eq!(&alice.name, "Alice");
        assert_eq!(&bob.name, "Bob");

        assert_eq!(alice.time, 1_565_545_684);
        assert_ne!(alice.time, bob.time);

        result
    });
}

#[test]
fn current_datetime_true() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
          time: DateTime
        }
    "#};

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Node {
        name: String,
        time: String,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Edge {
        node: Node,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UserCollection {
        edges: Vec<Edge>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        user_collection: UserCollection,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Result {
        data: Data,
    }

    with_mongodb(schema, |api| async move {
        let documents = json!([
            { "time": "2022-01-12T02:33:23.067Z", "name": "Alice" },
            { "time": "2022-01-12T02:33:23.067Z", "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { time: { currentDate: true } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name time } }  
              }
            }
        "};

        let result = api.execute(query).await;
        let mut deserialized: Result = serde_json::from_str(&result.as_json_string()).unwrap();

        let bob = deserialized.data.user_collection.edges.pop().unwrap().node;
        let alice = deserialized.data.user_collection.edges.pop().unwrap().node;

        assert_eq!(&alice.name, "Alice");
        assert_eq!(&bob.name, "Bob");

        assert_eq!(&alice.time, "2022-01-12T02:33:23.067Z");
        assert_ne!(alice.time, bob.time);

        result
    });
}

#[test]
fn inc_int() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { increment: 5 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 44
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn minimum_int() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { minimum: 36 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 36
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn maximum_int() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { maximum: 50 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 50
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn multiply_int() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Alice" },
            { "age": 39, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { multiply: 420 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 16380
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inc_float() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Float!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38.0, "name": "Alice" },
            { "age": 39.0, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { increment: 5.1 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38.0
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 44.1
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn minimum_float() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Float!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38.0, "name": "Alice" },
            { "age": 39.0, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { minimum: 36.1 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38.0
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 36.1
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn maximum_float() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Float!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38.0, "name": "Alice" },
            { "age": 39.0, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { maximum: 50.2 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38.0
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 50.2
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn multiply_float() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Float!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38.0, "name": "Alice" },
            { "age": 39.0, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { multiply: 420.2 } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": 38.0
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": 16387.8
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inc_bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: BigInt!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberLong": "38" }, "name": "Alice" },
            { "age": { "$numberLong": "39" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { increment: "5" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "44"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn minimum_bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: BigInt!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberLong": "38" }, "name": "Alice" },
            { "age": { "$numberLong": "39" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { minimum: "36" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "36"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn maximum_bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: BigInt!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberLong": "38" }, "name": "Alice" },
            { "age": { "$numberLong": "39" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { maximum: "50" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "50"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn multiply_bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: BigInt!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberLong": "38" }, "name": "Alice" },
            { "age": { "$numberLong": "39" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { multiply: "420" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "16380"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn inc_decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Decimal!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberDecimal": "38.0" }, "name": "Alice" },
            { "age": { "$numberDecimal": "39.0" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { increment: "5.0" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38.0"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "44.0"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn minimum_decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Decimal!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberDecimal": "38.0" }, "name": "Alice" },
            { "age": { "$numberDecimal": "39.0" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { minimum: "36.1" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38.0"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "36.1"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn maximum_decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Decimal!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberDecimal": "38.0" }, "name": "Alice" },
            { "age": { "$numberDecimal": "39.0" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { maximum: "50.1" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38.0"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "50.1"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn multiply_decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Decimal!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "$numberDecimal": "38.0" }, "name": "Alice" },
            { "age": { "$numberDecimal": "39.0" }, "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { age: { multiply: "420.2" } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name age } }  
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
                    "name": "Alice",
                    "age": "38.0"
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "age": "16387.80"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_add_to_set() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [6, 9], "name": "Alice" },
            { "numbers": [4, 2, 0], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { addToSet: { each: [4, 2, 0, 6, 9 ] } } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      6,
                      9
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      4,
                      2,
                      0,
                      6,
                      9
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_pop_first() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [6, 9], "name": "Alice" },
            { "numbers": [4, 2, 0], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { pop: FIRST } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      6,
                      9
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      2,
                      0
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_pop_last() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [6, 9], "name": "Alice" },
            { "numbers": [4, 2, 0], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { pop: LAST } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      6,
                      9
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      4,
                      2
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_pull() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [6, 9], "name": "Alice" },
            { "numbers": [4, 2, 0, 0], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { pull: { eq: 0 } } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      6,
                      9
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      4,
                      2
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_pull_nested() {
    let schema = indoc! {r#"
        type Inner {
          value: Int      
        }

        type User @model(connector: "test", collection: "users") {
          name: String!
          inner: [Inner!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "inner": [{ "value": 6 }, { "value": 9 }], "name": "Alice" },
            { "inner": [{ "value": 4 }, { "value": 2 }], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { inner: { pull: { value: { eq: 4 } } } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name inner { value} } }  
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
                    "name": "Alice",
                    "inner": [
                      {
                        "value": 6
                      },
                      {
                        "value": 9
                      }
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "inner": [
                      {
                        "value": 2
                      }
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_push() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [4, 2, 0], "name": "Alice" },
            { "numbers": [6, 9], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { push: { each: [1, 2, 3], sort: ASC, slice: 4, position: -1 } } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      4,
                      2,
                      0
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      1,
                      2,
                      3,
                      6
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn array_pull_all() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          numbers: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "numbers": [4, 2, 0], "name": "Alice" },
            { "numbers": [6, 9, 1, 2, 3], "name": "Bob" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userUpdateMany(
                filter: { name: { eq: "Bob" } },
                input: { numbers: { pullAll: [1, 2, 3] } }
              ) {
                matchedCount
                modifiedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userUpdateMany":{"matchedCount":1,"modifiedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { name numbers } }  
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
                    "name": "Alice",
                    "numbers": [
                      4,
                      2,
                      0
                    ]
                  }
                },
                {
                  "node": {
                    "name": "Bob",
                    "numbers": [
                      6,
                      9
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

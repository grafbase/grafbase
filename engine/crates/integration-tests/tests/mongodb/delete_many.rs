use expect_test::expect;
use indoc::indoc;
use integration_tests::{with_mongodb, with_namespaced_mongodb, GetPath};
use serde_json::json;

#[test]
fn empty() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let query = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { eq: 39 } }) {
                deletedCount
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userDeleteMany": {
              "deletedCount": 0
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "mongo", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_namespaced_mongodb("mongo", schema, |api| async move {
        let query = indoc! {r"
            mutation {
              mongo {
                userDeleteMany(filter: { age: { eq: 39 } }) {
                  deletedCount
                }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "mongo": {
              "userDeleteMany": {
                "deletedCount": 0
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn eq() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { eq: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 38
                  }
                },
                {
                  "node": {
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
fn renamed_eq() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int! @map(name: "renamed")
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "renamed": 38 },
            { "renamed": 39 },
            { "renamed": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { eq: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 38
                  }
                },
                {
                  "node": {
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
fn nested() {
    let schema = indoc! {r#"
        type Age {
          number: Int
        }

        type User @model(connector: "test", collection: "users") {
          age: Age!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "number": 38 } },
            { "age": { "number": 39 } },
            { "age": { "number": 40 } },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { number: { eq: 39 } } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age { number } } }  
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
                    "age": {
                      "number": 38
                    }
                  }
                },
                {
                  "node": {
                    "age": {
                      "number": 40
                    }
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_renamed() {
    let schema = indoc! {r#"
        type Age {
          number: Int @map(name: "renamed")
        }

        type User @model(connector: "test", collection: "users") {
          age: Age! @map(name: "renamed")
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "renamed": { "renamed": 38 } },
            { "renamed": { "renamed": 39 } },
            { "renamed": { "renamed": 40 } },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { number: { eq: 39 } } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age { number } } }  
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
                    "age": {
                      "number": 38
                    }
                  }
                },
                {
                  "node": {
                    "age": {
                      "number": 40
                    }
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nested_eq() {
    let schema = indoc! {r#"
        type B {
          c: String
        }

        type A {
          b: B
          d: String
        }
        
        type User @model(connector: "test", collection: "users") {
          data: A
          other: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "other": 1, "data": { "b": { "c": "test" }, "d": "other" } },
            { "other": 2, "data": { "b": { "c": "jest" }, "d": "brother" } },
            { "other": 3, "data": { "b": { "c": "gest" }, "d": "nothing" } },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: {
                data: {
                  b: { c: { eq: "test" } }
                  d: { eq: "other" }
                }
                other: { eq: 1 }
              }) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data { b { c } d } other} }  
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
                    "data": {
                      "b": {
                        "c": "jest"
                      },
                      "d": "brother"
                    },
                    "other": 2
                  }
                },
                {
                  "node": {
                    "data": {
                      "b": {
                        "c": "gest"
                      },
                      "d": "nothing"
                    },
                    "other": 3
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn ne() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { ne: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":2}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn gt() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { gt: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 38
                  }
                },
                {
                  "node": {
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn lt() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { lt: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 39
                  }
                },
                {
                  "node": {
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
fn gte() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { gte: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":2}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 38
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn lte() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { lte: 39 } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":2}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
fn r#in() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { in: [38, 40] } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":2}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nin() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 40 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: { age: { nin: [38, 40] } }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 38
                  }
                },
                {
                  "node": {
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
fn all() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Bob" },
            { "age": 39, "name": "Alice" },
            { "age": 39, "name": "Tim" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { ALL: [
                { age: { eq: 39 } },
                { name: { eq: "Alice" } }
              ]}) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

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
                    "name": "Bob",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Tim",
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn none() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Bob" },
            { "age": 39, "name": "Alice" },
            { "age": 39, "name": "Tim" },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { NONE: [
                { age: { eq: 38 } },
                { name: { eq: "Alice" } }
              ]}) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

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
                    "name": "Bob",
                    "age": 38
                  }
                },
                {
                  "node": {
                    "name": "Alice",
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn any() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38, "name": "Bob" },
            { "age": 39, "name": "Alice" },
            { "age": 39, "name": "Tim" },
            { "age": 40, "name": "Rachel" }
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: { ANY: [
                { age: { eq: 39 } },
                { name: { eq: "Bob" } }
              ]}) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":3}}}"#]];

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
fn not() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 38 },
            { "age": 39 },
            { "age": 39 },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: {
                age: { not: { eq: 39 } }
              }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { age } }  
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
                    "age": 39
                  }
                },
                {
                  "node": {
                    "age": 39
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date_eq() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          birthday: Date!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            {
                "birthday": {
                    "$date": {
                        "$numberLong": "1641945600000"
                    }
                },
            },
            {
                "birthday": {
                    "$date": {
                        "$numberLong": "1642945600000"
                    }
                },
            },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: {
                birthday: { eq: "2022-01-12" }
              }) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { birthday } }  
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
                    "birthday": "2022-01-23"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn datetime_eq() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          birthday: DateTime!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            {
                "birthday": {
                    "$date": "2022-01-12T02:33:23.067Z",
                },
            },
            {
                "birthday": {
                    "$date": "2022-01-12T02:33:23.067+04:00",
                },
            },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: {
                birthday: { eq: "2022-01-12T02:33:23.067+04:00" }
              }) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { birthday } }  
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
                    "birthday": "2022-01-12T02:33:23.067Z"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp_eq() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          registered: Timestamp!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            {
                "registered": {
                    "$timestamp": {
                        "t": 1_565_545_684,
                        "i": 1
                    }
                }
            },
            {
                "registered": {
                    "$timestamp": {
                        "t": 1_565_545_687,
                        "i": 1
                    }
                }
            },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: {
                registered: { eq: 1565545684 }
              }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { registered } }  
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
                    "registered": 1565545687
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn simple_array_all() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: [Int]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [4, 2, 0] },
            { "data": [2, 3, 4] },
            { "data": [6, 6, 6] }
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: {
                data: { all: [2, 3, 4] }
              }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data } }  
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
                    "data": [
                      4,
                      2,
                      0
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      6,
                      6,
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
fn simple_array_size() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: [Int]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [1] },
            { "data": [6, 9] },
            { "data": [4, 2, 0] }
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: {
                data: { size: 2 }
              }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data } }  
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
                    "data": [
                      1
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      4,
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
fn simple_array_elemmatch() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: [Int]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [1] },
            { "data": [6, 9] },
            { "data": [4, 2, 0] }
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r"
            mutation {
              userDeleteMany(filter: {
                data: { elemMatch: { eq: 1 } }
              }) {
                deletedCount
              }
            }
        "};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data } }  
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
                    "data": [
                      6,
                      9
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      4,
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
fn complex_array_elemmatch() {
    let schema = indoc! {r#"
        type Address {
          street: String @map(name: "street_name")
        }

        type User @model(connector: "test", collection: "users") {
          data: [Address]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [ { "street_name": "Ball" }] },
            { "data": [ { "street_name": "Wall" }] },
            { "data": [ { "street_name": "Gall" }] },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: {
                data: { elemMatch: { street: { eq: "Wall" } } }
              }) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data { street } } }  
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
                    "data": [
                      {
                        "street": "Ball"
                      }
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      {
                        "street": "Gall"
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
fn complex_double_nested_array_elemmatch() {
    let schema = indoc! {r#"
        type Street {
          name: String @map(name: "street_name")            
        }

        type Address {
          street: Street
        }

        type User @model(connector: "test", collection: "users") {
          data: [Address]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [ { "street": { "street_name": "Ball" } } ] },
            { "data": [ { "street": { "street_name": "Wall" } } ] },
            { "data": [ { "street": { "street_name": "Gall" } } ] },
        ]);

        api.insert_many("users", documents).await;

        let mutation = indoc! {r#"
            mutation {
              userDeleteMany(filter: {
                data: { elemMatch: { street: { name: { eq: "Wall" } } } }
              }) {
                deletedCount
              }
            }
        "#};

        let result = api.execute(mutation).await;
        let expected = expect![[r#"{"data":{"userDeleteMany":{"deletedCount":1}}}"#]];

        expected.assert_eq(&result.as_json_string());

        let query = indoc! {r"
            query {
              userCollection(first: 10) {
                edges { node { data { street { name } } } }  
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
                    "data": [
                      {
                        "street": {
                          "name": "Ball"
                        }
                      }
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      {
                        "street": {
                          "name": "Gall"
                        }
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

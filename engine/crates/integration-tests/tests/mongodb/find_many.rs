mod pagination;

use expect_test::expect;
use indoc::indoc;
use integration_tests::{with_mongodb, with_namespaced_mongodb};
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
            query {
              userCollection(first: 10, filter: { age: { eq: 39 } }) {
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
              "edges": []
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespaced() {
    let schema = indoc! {r#"
        type User @model(connector: "myMongo", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_namespaced_mongodb("myMongo", schema, |api| async move {
        let query = indoc! {r"
            query {
              myMongo {
                userCollection(first: 10, filter: { age: { eq: 39 } }) {
                  edges { node { age } }
                }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "myMongo": {
              "userCollection": {
                "edges": []
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn missing_first_or_last() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let query = indoc! {r"
            query {
              userCollection(filter: { age: { eq: 39 } }) {
                edges { node { age } }
              }
            }
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": null
          },
          "errors": [
            {
              "message": "please limit your selection by setting either the first or last parameter",
              "locations": [
                {
                  "line": 2,
                  "column": 3
                }
              ],
              "path": [
                "userCollection"
              ]
            }
          ]
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { eq: 39 } }) {
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

        let query = indoc! {r#"
            query {
              userCollection(
                filter: {
                  data: {
                    b: { c: { eq: "test" } }
                    d: { eq: "other" }
                  }
                  other: { eq: 1 }
                },
                first: 100
              ) {
                edges {
                  node {
                    data { b { c } d }
                    other
                  }
                }
              }
            }
        "#};

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
                        "c": "test"
                      },
                      "d": "other"
                    },
                    "other": 1
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { ne: 39 } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { gt: 39 } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { lt: 39 } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { gte: 39 } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { lte: 39 } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { in: [38, 40] } }) {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: { age: { nin: [38, 40] } }) {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { ALL: [
                { age: { eq: 39 } },
                { name: { eq: "Alice" } }
              ]}) {
                edges { node { name age } }
              }
            }
        "#};

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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { NONE: [
                { age: { eq: 38 } },
                { name: { eq: "Alice" } }
              ]}) {
                edges { node { name age } }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
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
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: { ANY: [
                { age: { eq: 39 } },
                { name: { eq: "Bob" } }
              ]}) {
                edges { node { name age } }
              }
            }
        "#};

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
fn not() {
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: {
                age: { not: { eq: 39 } }
              }) {
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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: {
                birthday: { eq: "2022-01-12" }
              }) {
                edges { node { birthday } }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "birthday": "2022-01-12"
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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: {
                birthday: { eq: "2022-01-12T02:33:23.067+04:00" }
              }) {
                edges { node { birthday } }
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "edges": [
                {
                  "node": {
                    "birthday": "2022-01-11T22:33:23.067Z"
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

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: {
                registered: { eq: 1565545684 }
              }) {
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
                    "registered": 1565545684
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
            { "data": [1, 2, 3] },
            { "data": [2, 3, 4] },
            { "data": [6, 6, 6] }
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: {
                data: { all: [2, 3, 4] }
              }) {
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
                      2,
                      3,
                      4
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
            { "data": [1, 2] },
            { "data": [1, 2, 3] }
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: {
                data: { size: 2 }
              }) {
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
                      1,
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
fn simple_array_elemmatch() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: [Int]
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "data": [1] },
            { "data": [1, 2] },
            { "data": [1, 2, 3] }
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10, filter: {
                data: { elemMatch: { eq: 2 } }
              }) {
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
                      1,
                      2
                    ]
                  }
                },
                {
                  "node": {
                    "data": [
                      1,
                      2,
                      3
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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: {
                data: { elemMatch: { street: { eq: "Wall" } } }
              }) {
                edges { node { data { street }} }
              }
            }
        "#};

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
                        "street": "Wall"
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

        let query = indoc! {r#"
            query {
              userCollection(first: 10, filter: {
                data: { elemMatch: { street: { name: { eq: "Wall" } } } }
              }) {
                edges { node { data { street { name } } } }
              }
            }
        "#};

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
                          "name": "Wall"
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

#[test]
fn simple_sort_asc() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 39 },
            { "age": 40 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 10,
                filter: { age: { gt: 38 } },
                orderBy: [{ age: ASC }]
              ) {
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
fn simple_sort_desc() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 39 },
            { "age": 40 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 10,
                filter: { age: { gt: 38 } },
                orderBy: [{ age: DESC }]
              ) {
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
fn nested_sort() {
    let schema = indoc! {r#"
        type Age {
          number: Int! @map(name: "real_number")
        }

        type User @model(connector: "test", collection: "users") {
          age: Age!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": { "real_number": 40 } },
            { "age": { "real_number": 38 } },
            { "age": { "real_number": 39 } },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 10,
                filter: { age: { number: { gt: 38 } } },
                orderBy: [{ age: { number: ASC } }]
              ) {
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
                      "number": 39
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

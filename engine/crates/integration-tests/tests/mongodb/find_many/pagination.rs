use expect_test::expect;
use indoc::{formatdoc, indoc};
use integration_tests::{with_mongodb, GetPath};
use serde_json::json;

#[test]
fn after_with_no_sort() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 2) { pageInfo { endCursor } }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.endCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(first: 2, after: "{cursor}") {{ edges {{ node {{ age }} }} }}
            }}
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
fn after_with_sort_asc_no_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 2,
                orderBy: [{ age: ASC }]
              ) { 
                edges { node { age } }
                pageInfo { endCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.endCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                first: 2,
                orderBy: [{{ age: ASC }}],
                after: "{cursor}") {{
                  edges {{ node {{ age }} }}
                }}
            }}
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
fn after_with_sort_asc_with_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": null },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 1,
                orderBy: [{ age: ASC }]
              ) {
                edges { node { age } }
                pageInfo { endCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.endCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                first: 2,
                orderBy: [{{ age: ASC }}],
                after: "{cursor}"
              ) {{ edges {{ node {{ age }} }} }}
            }}
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
fn after_with_sort_desc_no_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 38 },
            { "age": 39 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 2,
                orderBy: [{ age: DESC }]
              ) { 
                edges { node { age } }
                pageInfo { endCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.endCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                first: 2,
                orderBy: [{{ age: DESC }}],
                after: "{cursor}") {{
                  edges {{ node {{ age }} }}
                }}
            }}
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
fn after_with_sort_desc_with_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": null },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                first: 2,
                orderBy: [{ age: DESC }]
              ) {
                edges { node { age } }
                pageInfo { endCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.endCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                first: 2,
                orderBy: [{{ age: DESC }}],
                after: "{cursor}"
              ) {{ edges {{ node {{ age }} }} }}
            }}
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
                    "age": null
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn after_has_next_and_previous_page_if_more_data() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 2) { pageInfo { hasPreviousPage hasNextPage } }
            }     
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": true
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn after_has_next_and_previous_page_if_not_more_data() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(first: 3) { pageInfo { hasPreviousPage hasNextPage } }
            }     
        "};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "userCollection": {
              "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn before_with_no_sort() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(last: 2) { edges { node { age } } pageInfo { startCursor } }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.startCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(last: 1, before: "{cursor}") {{ edges {{ node {{ age }} }} }}
            }}
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
fn before_with_sort_asc_no_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                last: 2,
                orderBy: [{ age: ASC }]
              ) {
                edges { node { age } }
                pageInfo { startCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.startCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                last: 1,
                orderBy: [{{ age: ASC }}],
                before: "{cursor}"
              ) {{
                edges {{ node {{ age }} }}
              }}
            }}
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
fn before_with_sort_asc_with_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
          name: String
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": null, "name": "Bob" },
            { "age": null, "name": "Alice" },
            { "age": null, "name": "Richard" },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                last: 2,
                orderBy: [{ age: ASC }, { name: ASC }]
              ) {
                edges { node { name age } }
                pageInfo { startCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.startCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                last: 1,
                orderBy: [{{ age: ASC }}, {{ name: ASC }}],
                before: "{cursor}"
              ) {{
                edges {{ node {{ name age }} }}
              }}
            }}
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
                    "age": null
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn before_with_sort_desc_no_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                last: 2,
                orderBy: [{ age: DESC }]
              ) {
                edges { node { age } }
                pageInfo { startCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.startCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                last: 1,
                orderBy: [{{ age: DESC }}],
                before: "{cursor}"
              ) {{
                edges {{ node {{ age }} }}
              }}
            }}
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
fn before_with_sort_desc_with_nulls() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
          name: String
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": null, "name": "Bob" },
            { "age": null, "name": "Alice" },
            { "age": null, "name": "Richard" },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(
                last: 2,
                orderBy: [{ age: DESC }, { name: DESC }]
              ) {
                edges { node { name age } }
                pageInfo { startCursor }
              }
            }     
        "};

        let response = api.execute(query).await;
        let cursor = response.get_string("userCollection.pageInfo.startCursor");

        let query = formatdoc! {r#"
            query {{
              userCollection(
                last: 1,
                orderBy: [{{ age: DESC }}, {{ name: DESC }}],
                before: "{cursor}"
              ) {{
                edges {{ node {{ name age }} }}
              }}
            }}
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
                    "name": "Richard",
                    "age": null
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn before_has_previous_page_if_more_data() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(last: 2) { edges { node { age } } pageInfo { hasPreviousPage hasNextPage } }
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
                    "age": 38
                  }
                }
              ],
              "pageInfo": {
                "hasPreviousPage": true,
                "hasNextPage": false
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn before_has_previous_page_if_not_more_data() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          age: Int
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let documents = json!([
            { "age": 40 },
            { "age": 39 },
            { "age": 38 },
        ]);

        api.insert_many("users", documents).await;

        let query = indoc! {r"
            query {
              userCollection(last: 3) { edges { node { age } } pageInfo { hasPreviousPage hasNextPage } }
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
                },
                {
                  "node": {
                    "age": 38
                  }
                }
              ],
              "pageInfo": {
                "hasPreviousPage": false,
                "hasNextPage": false
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

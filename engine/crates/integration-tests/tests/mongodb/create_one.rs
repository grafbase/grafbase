use expect_test::expect;
use indoc::indoc;
use integration_tests::{with_mongodb, with_namespaced_mongodb, GetPath};
use serde_json::json;

#[test]
fn only_implicit_fields() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String
        }
    "#};

    with_mongodb(schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              userCreate(input: {}) { insertedId }
            }         
        "};

        let result = api.execute(mutation).await;
        let inserted_id = result.get_string_opt("userCreate.insertedId");

        assert!(inserted_id.is_ok());

        result
    });
}

#[test]
fn namespacing() {
    let schema = indoc! {r#"
        type User @model(connector: "mongo", collection: "users") {
          name: String
        }
    "#};

    with_namespaced_mongodb("mongo", schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              mongo {
                userCreate(input: {}) { insertedId }
              }
            }         
        "};

        let result = api.execute(mutation).await;
        let inserted_id = result.get_string_opt("mongo.userCreate.insertedId");

        assert!(inserted_id.is_ok());

        result
    });
}

#[test]
fn capital_namespacing() {
    let schema = indoc! {r#"
        type User @model(connector: "Mongo", collection: "users") {
          name: String
        }
    "#};

    with_namespaced_mongodb("Mongo", schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              mongo {
                userCreate(input: {}) { insertedId }
              }
            }         
        "};

        let result = api.execute(mutation).await;
        let inserted_id = result.get_string_opt("mongo.userCreate.insertedId");

        assert!(inserted_id.is_ok());

        result
    });
}

#[test]
fn nested_data() {
    let schema = indoc! {r#"
        type Address {
          street: String! @map(name: "street_name")
          city: String!
        }

        type User @model(connector: "test", collection: "users") {
          address: Address! @map(name: "address_data")
        }
    "#};

    with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { address: { street: "Wall", city: "New York" }}) { insertedId }
            }         
        "#};

        let response = api.execute(mutation).await;
        let projection = json!({ "address_data.street_name": 1, "address_data.city": 1, "_id": 0 });
        let all = api.fetch_all("users", projection).await;

        let expected = expect![[r#"
            [
              {
                "address_data": {
                  "city": "New York",
                  "street_name": "Wall"
                }
              }
            ]"#]];

        let actual = serde_json::to_string_pretty(&all.documents).unwrap();
        expected.assert_eq(&actual);

        response
    });
}

#[test]
fn binary() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: Bytes!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { data: "e67803a39588be8a95731a21e27d7391"}) { insertedId }
            }         
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": "e67803a39588be8a95731a21e27d7391"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn date() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: Date!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { data: "2022-01-12"}) { insertedId }
            }         
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": "2022-01-12"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn datetime() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: DateTime!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { data: "2022-01-12T02:33:23.067Z"}) { insertedId }
            }         
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": "2022-01-12T02:33:23.067Z"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: Decimal!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { data: "3.14"}) { insertedId }
            }         
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": "3.14"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: BigInt!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r#"
            mutation {
              userCreate(input: { data: "9223372036854775807" }) { insertedId }
            }         
        "#};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": "9223372036854775807"
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn timestamp() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          data: Timestamp!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              userCreate(input: { data: 1565545664 }) { insertedId }
            }         
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data } } }
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
                    "data": 1565545664
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

#[test]
fn complex_array() {
    let schema = indoc! {r#"
        type Data {
          value: Int! @map(name: "renamed")
        }

        type User @model(connector: "test", collection: "users") {
          data: [Data!]!
        }
    "#};

    let result = with_mongodb(schema, |api| async move {
        let mutation = indoc! {r"
            mutation {
              userCreate(input: { data: [{ value: 123 }] }) { insertedId }
            }         
        "};

        api.execute(mutation).await;

        let query = indoc! {r"
            query {
              userCollection(first: 10) { edges { node { data { value } } } }
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
                        "value": 123
                      }
                    ]
                  }
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&result);
}

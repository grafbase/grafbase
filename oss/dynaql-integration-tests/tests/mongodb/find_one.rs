use dynaql_integration_tests::{with_mongodb, with_namespaced_mongodb};
use expect_test::expect;
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
            query {
              user(by: { id: "5ca4bbc7a2dd94ee5816238d" }) {
                id
                name
              }
            }
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": null
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

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                name
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Bob"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn namespacing() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_namespaced_mongodb("myMongo", schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              myMongo {{
                user(by: {{ id: "{id}" }}) {{
                  name
                }}
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "myMongo": {
              "user": {
                "name": "Bob"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn capital_namespacing() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_namespaced_mongodb("Mongo", schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              mongo {{
                user(by: {{ id: "{id}" }}) {{
                  name
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
              "user": {
                "name": "Bob"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn snake_namespacing() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_namespaced_mongodb("mong_o", schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              mongO {{
                user(by: {{ id: "{id}" }}) {{
                  name
                }}
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "mongO": {
              "user": {
                "name": "Bob"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn field_mapping() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String! @map(name: "real_name")
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "real_name": "Bob",
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                name
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Bob"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn nesting() {
    let schema = indoc! {r#"
        type Address {
          street: String! @map(name: "street_name")
          city: String!
        }

        type User @model(connector: "test", collection: "users") {
          address: Address! @map(name: "real_address")
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "real_address": {
                "street_name": "Wall",
                "city": "Street",
            }
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                address {{ street city }}
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "address": {
                "street": "Wall",
                "city": "Street"
              }
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn fragment() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          name: String!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "name": "Bob",
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            fragment Person on User {{
              name
            }}

            query {{
              user(by: {{ id: "{id}" }}) {{
                ...Person
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "name": "Bob"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn date() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          birthday: Date!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "birthday": {
                "$date": {
                    "$numberLong": "1641945600000"
                }
            },
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                birthday
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "birthday": "2022-01-12"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn datetime() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          birthday: DateTime!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "birthday": {
                "$date": "2022-01-12T02:33:23.067+00:00",
            },
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                birthday
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "birthday": "2022-01-12T02:33:23.067Z"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn timestamp() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          registered: Timestamp!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "registered": {
                "$timestamp": {
                    "t": 1_565_545_684,
                    "i": 1
                }
            },
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                registered
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "registered": 1565545684
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bytes() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          secrets: Bytes!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "secrets": {
                "$binary": {
                    "base64": "e67803a39588be8a95731a21e27d7391",
                    "subType": "05"
                }
            },
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                secrets
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "secrets": "e67803a39588be8a95731a21e27d7391"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn boolean() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          yes: Boolean!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "yes": true
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                yes
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "yes": true
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn float() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          num: Float!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "num": 1.23
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                num
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "num": 1.23
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn decimal() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          dec: Decimal!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "dec": {
                "$numberDecimal": "1.2345"
            }
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                dec
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "dec": "1.2345"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn bigint() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          num: BigInt!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "num": {
                "$numberLong": "9223372036854775807"
            }
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                num
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "num": "9223372036854775807"
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn simple_array() {
    let schema = indoc! {r#"
        type User @model(connector: "test", collection: "users") {
          ints: [Int!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "ints": [1, 2, 3]
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                ints
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "ints": [
                1,
                2,
                3
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

#[test]
fn complex_array() {
    let schema = indoc! {r#"
        type Data {
          value: Int! @map(name: "renamed")
        }

        type User @model(connector: "test", collection: "users") {
          ints: [Data!]!
        }
    "#};

    let response = with_mongodb(schema, |api| async move {
        let document = json!({
            "ints": [
                { "renamed": 1 },
                { "renamed": 2 },
                { "renamed": 3 }
            ]
        });

        let id = api.create_one("users", document).await.inserted_id;

        let query = formatdoc! {r#"
            query {{
              user(by: {{ id: "{id}" }}) {{
                ints {{ value }}
              }}
            }}
        "#};

        api.execute(query).await
    });

    let expected = expect![[r#"
        {
          "data": {
            "user": {
              "ints": [
                {
                  "value": 1
                },
                {
                  "value": 2
                },
                {
                  "value": 3
                }
              ]
            }
          }
        }"#]];

    expected.assert_eq(&response);
}

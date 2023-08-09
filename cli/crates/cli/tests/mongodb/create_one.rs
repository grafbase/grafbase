use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn with_id_and_mapped_string() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_request = json!({
        "document": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
            "real_name": "Jack",
        }
    });

    let server = Server::create_one(&config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            id: "5ca4bbc7a2dd94ee5816238d",
            name: "Jack"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_nested_data() {
    let config = indoc::formatdoc! {r#"
        type Address {{
          street: String! @map(name: "street_name")
          city: String!
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          address: Address! @map(name: "address_data")
        }}
    "#};

    let expected_request = json!({
        "document": {
            "address_data": {
                "street_name": "Wall",
                "city": "New York"
            }
        }
    });

    let server = Server::create_one(&config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            address: {
              street: "Wall",
              city: "New York"
            }
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_binary_data() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: Bytes!
        }}
    "#};

    let expected_request = json!({
        "document": {
            "data": {
                "$binary": {
                    "base64": "e67803a39588be8a95731a21e27d7391",
                    "subType": "05",
                },
            },
        }
    });

    let server = Server::create_one(config, "users", expected_request).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            data: "e67803a39588be8a95731a21e27d7391"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_date() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          date: Date!
        }}
    "#};

    let document = json!({
        "document": {
            "date": {
                "$date": {
                    "$numberLong": "1641945600000",
                },
            },
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            date: "2022-01-12"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_datetime() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          date: DateTime!
        }}
    "#};

    let document = json!({
        "document": {
            "date": {
                "$date": "2022-01-12T02:33:23.067+00:00"
            },
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            date: "2022-01-12T02:33:23.067+00:00"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_decimal() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          dec: Decimal!
        }}
    "#};

    let document = json!({
        "document": {
            "dec": {
                "$numberDecimal": "1.2345",
            },
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            dec: "1.2345"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_bigint() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          num: BigInt!
        }}
    "#};

    let document = json!({
        "document": {
            "num": {
                "$numberLong": "9223372036854775807",
            },
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            num: "9223372036854775807"
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_timestamp() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          time: Timestamp!
        }}
    "#};

    let document = json!({
        "document": {
            "time": {
                "$timestamp": {
                    "t": 1_565_545_664,
                    "i": 1
                }
            }
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            time: 1565545664
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_boolean() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          truth: Boolean!
        }}
    "#};

    let document = json!({
        "document": {
            "truth": true
        }
    });

    let server = Server::create_one(config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            truth: true
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_float() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          num: Float!
        }}
    "#};

    let document = json!({
        "document": {
            "num": 1.23
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            num: 1.23
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_simple_array() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          ints: [Int!]!
        }}
    "#};

    let document = json!({
        "document": {
            "ints": [1, 2, 3]
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            ints: [1, 2, 3]
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn with_complex_array() {
    let config = indoc::formatdoc! {r#"
        type Data {{
            value: Int! @map(name: "renamed")
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          ints: [Data!]!
        }}
    "#};

    let document = json!({
        "document": {
            "ints": [{ "renamed": 1 }, { "renamed": 2 }, { "renamed": 3 }]
        }
    });

    let server = Server::create_one(&config, "users", document).await;

    let response = server.request(indoc::indoc! {r#"
        mutation {
          userCreate(input: {
            ints: [{ value: 1 }, { value: 2 }, { value: 3 }]
          }) {
            insertedId
          }
        }"#
    });

    insta::assert_json_snapshot!(response.await, @r###"
    {
      "data": {
        "userCreate": {
          "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }
      }
    }
    "###);
}

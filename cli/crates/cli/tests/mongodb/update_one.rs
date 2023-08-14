use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn single_set() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$set": { "real_name": "Derp" }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { name: { set: "Derp" } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn single_unset() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$unset": { "real_name": true }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { name: { unset: true } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn unset_false() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String @map(name: "real_name")
        }}
    "#};

    let expected_body = json!({});

    let mut server = Server::update_one(&config, "users", expected_body).await;
    server.expected_requests(0);

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { name: { unset: false } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 0,
          "modifiedCount": 0
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_timestamp() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          registered: Timestamp
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$currentDate": { "registered": { "$type": "timestamp" } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { registered: { currentDate: true } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_datetime() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          registered: DateTime
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$currentDate": { "registered": { "$type": "date" } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { registered: { currentDate: true } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn current_datetime_false_triggering_empty_update() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          registered: DateTime
        }}
    "#};

    let expected_body = json!({});

    let mut server = Server::update_one(&config, "users", expected_body).await;
    server.expected_requests(0);

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { registered: { currentDate: false } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 0,
          "modifiedCount": 0
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn inc_int() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          number: Int
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$inc": { "number": 420 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { number: { increment: 420 } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn minimum_int() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          number: Int
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$min": { "number": 420 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { number: { minimum: 420 } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn maximum_int() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          number: Int
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$max": { "number": 420 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { number: { maximum: 420 } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn multiply_int() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          number: Int
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$mul": { "number": 420 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { number: { multiply: 420 } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_add_to_set() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$addToSet": { "numbers": { "$each": [1, 2, 3] } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { addToSet: { each: [1, 2, 3] } } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_pop_first() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$pop": { "numbers": -1 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { pop: FIRST } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_pop_last() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$pop": { "numbers": 1 }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { pop: LAST } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_pull() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$pull": { "numbers": { "$eq": 5 } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { pull: { eq: 5 } } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_pull_nested() {
    let config = indoc::formatdoc! {r#"
        type Inner {{
          value: Int
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          inner: [Inner]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$pull": { "inner": { "value": { "$eq": 5 } } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { inner: { pull: { value: { eq: 5 } } } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_push() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$push": { "numbers": { "$each": [1, 2, 3], "$position": -1, "$slice": 10, "$sort": 1 } }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { push: { each: [1, 2, 3], sort: ASC, slice: 10, position: -1 } } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn array_pull_all() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          numbers: [Int]
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$pullAll": { "numbers": [1, 2, 3] }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { numbers: { pullAll: [1, 2, 3] } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn combining_operations() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int
          name: String
        }}
    "#};

    let expected_body = json!({
        "filter": {
            "_id": { "$oid": "5ca4bbc7a2dd94ee5816238d" },
        },
        "update": {
            "$set": { "age": 30, "name": "Bob" }
        }
    });

    let server = Server::update_one(&config, "users", expected_body).await;

    let request = server.request(indoc::indoc! {r#"
        mutation {
          userUpdate(
            by: { id: "5ca4bbc7a2dd94ee5816238d" },
            input: { age: { set: 30 }, name: { set: "Bob" } }
          ) {
            matchedCount
            modifiedCount
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userUpdate": {
          "matchedCount": 1,
          "modifiedCount": 1
        }
      }
    }
    "###);
}

use super::MONGODB_CONNECTOR;
use crate::Server;
use serde_json::json;
use wiremock::ResponseTemplate;

#[tokio::test(flavor = "multi_thread")]
async fn id_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [{
            "_id": "5ca4bbc7a2dd94ee5816238d",
        }]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "5ca4bbc7a2dd94ee5816238d"
              }
            }
          ]
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn cursor() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [{
            "_id": "5ca4bbc7a2dd94ee5816238d",
        }]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } cursor }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "5ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            }
          ]
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn end_cursor() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$ne": { "$oid": "7ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "5ca4bbc7a2dd94ee5816238d",
            },
            {
                "_id": "6ca4bbc7a2dd94ee5816238d"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } cursor } pageInfo { endCursor }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "5ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            },
            {
              "node": {
                "id": "6ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            }
          ],
          "pageInfo": {
            "endCursor": "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          }
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn start_cursor() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$ne": { "$oid": "7ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "5ca4bbc7a2dd94ee5816238d",
            },
            {
                "_id": "6ca4bbc7a2dd94ee5816238d"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } cursor } pageInfo { startCursor }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "5ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            },
            {
              "node": {
                "id": "6ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            }
          ],
          "pageInfo": {
            "startCursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          }
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn after_id() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                { "_id": { "$ne": { "$oid": "7ca4bbc7a2dd94ee5816238d" } } },
                { "_id": { "$gt": { "$oid": "5ca4bbc7a2dd94ee5816238d" } } }
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 100,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "6ca4bbc7a2dd94ee5816238d"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } },
            after: "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          ) {
            edges { node { id } cursor } pageInfo { startCursor }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "6ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            }
          ],
          "pageInfo": {
            "startCursor": "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          }
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn before_id() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                { "_id": { "$ne": { "$oid": "7ca4bbc7a2dd94ee5816238d" } } },
                { "_id": { "$lt": { "$oid": "6ca4bbc7a2dd94ee5816238d" } } }
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 100,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "5ca4bbc7a2dd94ee5816238d"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } },
            before: "NmNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          ) {
            edges { node { id } cursor } pageInfo { startCursor }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": [
            {
              "node": {
                "id": "5ca4bbc7a2dd94ee5816238d"
              },
              "cursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
            }
          ],
          "pageInfo": {
            "startCursor": "NWNhNGJiYzdhMmRkOTRlZTU4MTYyMzhk"
          }
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn simple_sort() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "sort": {
            "_id": 1,
            "name": -1,
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, orderBy: { id: ASC, name: DESC }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn nested_sort() {
    let config = indoc::formatdoc! {r#"
        type Address {{
          street: String @map(name: "street_name")
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          address: Address
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "sort": {
            "address.street_name": 1,
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, orderBy: { address: { street: ASC } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_ne() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$ne": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "5ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_gt() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$gt": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { gt: "5ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_gte() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$gte": { "$oid": "5ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { gte: "5ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_lt() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$lt": { "$oid": "6ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { lt: "6ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_lte() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": { "$lte": { "$oid": "6ca4bbc7a2dd94ee5816238d" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { lte: "6ca4bbc7a2dd94ee5816238d" } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_in() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": {
                "$in": [
                    { "$oid": "5ca4bbc7a2dd94ee5816238d" },
                    { "$oid": "6ca4bbc7a2dd94ee5816238d" }
                ]
            },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { in: ["5ca4bbc7a2dd94ee5816238d", "6ca4bbc7a2dd94ee5816238d"] } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_nin() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "_id": {
                "$nin": [
                    { "$oid": "5ca4bbc7a2dd94ee5816238d" },
                    { "$oid": "6ca4bbc7a2dd94ee5816238d" }
                ]
            },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { nin: ["5ca4bbc7a2dd94ee5816238d", "6ca4bbc7a2dd94ee5816238d"] } }) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_and() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                { "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } } },
                { "_id": { "$eq": { "$oid": "6ca4bbc7a2dd94ee5816238d" } } },
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { ALL: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}
#[tokio::test(flavor = "multi_thread")]
async fn id_none() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$nor": [
                { "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } } },
                { "_id": { "$eq": { "$oid": "6ca4bbc7a2dd94ee5816238d" } } },
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { NONE: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_any() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$or": [
                { "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } } },
                { "_id": { "$eq": { "$oid": "6ca4bbc7a2dd94ee5816238d" } } },
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { ANY: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}) {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn id_not() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$not": { "_id": { "$eq": { "$oid": "5ca4bbc7a2dd94ee5816238d" } } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { NOT: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn string_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {
            "name": { "$eq": "Bob" },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { name: { eq: "Bob" } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn int_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int
        }}
    "#};

    let body = json!({
        "filter": {
            "age": { "$eq": 18 },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: 18 } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn float_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Float
        }}
    "#};

    let body = json!({
        "filter": {
            "age": { "$eq": 18.1 },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: 18.1 } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn bool_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Boolean
        }}
    "#};

    let body = json!({
        "filter": {
            "age": { "$eq": true },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: true } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn date_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          date: Date
        }}
    "#};

    let body = json!({
        "filter": {
            "date": { "$eq": { "$date": { "$numberLong": "1641945600000" }} },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { date: { eq: "2022-01-12" } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn datetime_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          date: DateTime
        }}
    "#};

    let body = json!({
        "filter": {
            "date": { "$eq": { "$date": { "$numberLong": "1641954803067" } } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { date: { eq: "2022-01-12T02:33:23.067+00:00" } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn decimal_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          num: Decimal
        }}
    "#};

    let body = json!({
        "filter": {
            "num": { "$eq": { "$numberDecimal": "1.2345" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { num: { eq: "1.2345" } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn bigint_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          num: BigInt
        }}
    "#};

    let body = json!({
        "filter": {
            "num": { "$eq": { "$numberLong": "9223372036854775807" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { num: { eq: "9223372036854775807" } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn timestamp_eq() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          time: Timestamp
        }}
    "#};

    let body = json!({
        "filter": {
            "time": { "$eq": { "$timestamp": { "t": 1_565_545_664, "i": 1 } } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { time: { eq: 1565545664 } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn simple_array_all() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: [Int]
        }}
    "#};

    let body = json!({
        "filter": {
            "data": { "$all": [1, 2, 3] }
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { all: [1, 2, 3] } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn simple_array_size() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: [Int]
        }}
    "#};

    let body = json!({
        "filter": {
            "data": { "$size": 3 }
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { size: 3 } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn simple_array_elematch() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: [Int]
        }}
    "#};

    let body = json!({
        "filter": {
            "data": { "$elemMatch": { "$eq": 2 } }
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { elemMatch: { eq: 2 } } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn complex_array_elematch() {
    let config = indoc::formatdoc! {r#"
        type Address {{
          street: String @map(name: "street_name")
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: [Address]
        }}
    "#};

    let body = json!({
        "filter": {
            "data": { "$elemMatch": { "street_name": { "$eq": "Wall" }} }
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { elemMatch: { street: { eq: "Wall" } } } })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn double_nested_array_elematch() {
    let config = indoc::formatdoc! {r#"
        type Street {{
          name: String @map(name: "street_name")
        }}

        type Address {{
          street: Street
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: [Address]
        }}
    "#};

    let body = json!({
        "filter": {
            "data": { "$elemMatch": { "street.street_name": { "$eq": "Wall" }} }
        },
        "projection": {
            "_id": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: {
            data: {
              elemMatch: {
                street: { name: { eq: "Wall" } }
              }
            }
          })
          {
            edges { node { id } }
          }
        }   
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn nested_eq() {
    let config = indoc::formatdoc! {r#"
        type B {{
          c: String
        }}

        type A {{
          b: B
          d: String
        }}

        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          data: A
          other: Int
        }}
    "#};

    let body = json!({
        "filter": {
            "data.b.c": { "$eq": "test" },
            "data.d": { "$eq": "other"},
            "other": { "$eq": 1 }
        },
        "projection": {
            "_id": 1,
            "data.b.c": 1,
            "data.d": 1,
            "other": 1
        },
        "limit": 100
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            filter: {
              data: {
                b: { c: { eq: "test" } }
                d: { eq: "other" }
              }
              other: { eq: 1 }
            }
          ) {
            edges {
              node {
                data {
                  b {
                    c
                  }
                  d
                }
                other
              }
            }
          }
        }
    "#});

    insta::assert_json_snapshot!(request.await, @r###"
    {
      "data": {
        "userCollection": {
          "edges": []
        }
      }
    }
    "###);
}

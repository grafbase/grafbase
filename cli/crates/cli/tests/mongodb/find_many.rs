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
        "limit": 101
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
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
            "_id": { "$eq": { "$oid": "64c7a09da6591eea08725b7b" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 101
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [{
            "_id": "64c7a09da6591eea08725b7b",
        }]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "64c7a09da6591eea08725b7b" } }, first: 100) {
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
                "id": "64c7a09da6591eea08725b7b"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
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
        "limit": 101
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64c7a09da6591eea08725b7b",
            },
            {
                "_id": "64c7dc46b73a048947eebd55"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
                "id": "64c7a09da6591eea08725b7b"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            },
            {
              "node": {
                "id": "64c7dc46b73a048947eebd55"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            }
          ],
          "pageInfo": {
            "endCursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
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
        "limit": 101
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64c7a09da6591eea08725b7b",
            },
            {
                "_id": "64c7dc46b73a048947eebd55"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "7ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
                "id": "64c7a09da6591eea08725b7b"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            },
            {
              "node": {
                "id": "64c7dc46b73a048947eebd55"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            }
          ],
          "pageInfo": {
            "startCursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
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
                { "_id": { "$eq": { "$oid": "64c7a09da6591eea08725b7b" } } },
                { "_id": { "$gt": { "$oid": "64c7dc46b73a048947eebd55" } } }
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 101,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64c7a09da6591eea08725b7b"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            filter: { id: { eq: "64c7a09da6591eea08725b7b" } },
            after: "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ",
            first: 100
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
                "id": "64c7a09da6591eea08725b7b"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            }
          ],
          "pageInfo": {
            "startCursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
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
                { "_id": { "$eq": { "$oid": "64c7a09da6591eea08725b7b" } } },
                { "_id": { "$lt": { "$oid": "64c7dc46b73a048947eebd55" } } }
            ]
        },
        "projection": {
            "_id": 1
        },
        "limit": 101,
    });

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64c7a09da6591eea08725b7b"
            }
        ]
    }));

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(response);

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            filter: { id: { eq: "64c7a09da6591eea08725b7b" } },
            before: "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ",
            first: 100
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
                "id": "64c7a09da6591eea08725b7b"
              },
              "cursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
            }
          ],
          "pageInfo": {
            "startCursor": "ZmllbGRzAG5hbWUAA19pZAB2YWx1ZQBPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhRkaXJlY3Rpb24ACUFzY2VuZGluZwADFlFIAwEDEVEgFBQkAQckAWcBAQEHKAIkAQ"
          }
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn cursor_after_non_null_one_sort_column_ascending() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int!
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": 21
                                    }
                                },
                                {
                                    "_id": {
                                        "$gt": {
                                            "$oid": "64cbb12e1ccbe9c84921db52"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "age": {
                                "$gt": 21
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "age": 1
        },
        "sort": {
            "age": 1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { age: ASC} ],
            after: "ZmllbGRzAG5hbWUAA2FnZQB2YWx1ZQBQb3NJbnQAAQgBAQEVCGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWNSwDAQMRNSAUFCQDX2lkAE9iamVjdElkABg2NGNiYjEyZTFjY2JlOWM4NDkyMWRiNTIAASQBAQEeFAlBc2NlbmRpbmcAA118cwMBAxFBFhQUJAJOCCQkAZQBAQEJKAIkAQ",
            first: 100
          ) {
            edges { node { id age } }
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
async fn cursor_before_non_null_one_sort_column_ascending() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int!
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": 26
                                    }
                                },
                                {
                                    "_id": {
                                        "$lt": {
                                            "$oid": "64c91fa4ba621ed5a297b48f"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "$or": [
                                {
                                    "age": {
                                        "$lt": 26
                                    }
                                },
                                {
                                    "age": {
                                        "$eq": null
                                    }
                                }
                            ]
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "age": 1
        },
        "sort": {
            "age": 1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { age: ASC} ],
            before: "ZmllbGRzAG5hbWUAA2FnZQB2YWx1ZQBQb3NJbnQAAQgBAQEaCGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWNSwDAQMRNSAUFCQDX2lkAE9iamVjdElkABg2NGM5MWZhNGJhNjIxZWQ1YTI5N2I0OGYAASQBAQEeFAlBc2NlbmRpbmcAA118cwMBAxFBFhQUJAJOCCQkAZQBAQEJKAIkAQ",
            first: 100
          ) {
            edges { node { id age } }
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
async fn cursor_before_non_null_one_sort_column_descending() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int!
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": 32
                                    }
                                },
                                {
                                    "_id": {
                                        "$lt": {
                                            "$oid": "64c7dc46b73a048947eebd55"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "age": {
                                "$gt": 32
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "age": 1
        },
        "sort": {
            "age": -1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { age: DESC } ],
            before: "ZmllbGRzAG5hbWUAA2FnZQB2YWx1ZQBQb3NJbnQAAQgBAQEgCGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzYtAwEDEjYhFBQkA19pZABPYmplY3RJZAAYNjRjN2RjNDZiNzNhMDQ4OTQ3ZWViZDU1AAEkAQEBHhQJQXNjZW5kaW5nAANefXQDAQMRQRYUFCQCTggkJAGVAQEBCSgCJAE",
            first: 100
          ) {
            edges { node { id age } }
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
async fn cursor_after_non_null_one_sort_column_descending() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          age: Int!
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": 40
                                    }
                                },
                                {
                                    "_id": {
                                        "$gt": {
                                            "$oid": "64c92549ba621ed5a297b490"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "$or": [
                                {
                                    "age": {
                                        "$lt": 40
                                    }
                                },
                                {
                                    "age": {
                                        "$eq": null
                                    }
                                }
                            ]
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "age": 1
        },
        "sort": {
            "age": -1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { age: DESC } ],
            after: "ZmllbGRzAG5hbWUAA2FnZQB2YWx1ZQBQb3NJbnQAAQgBAQEoCGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzYtAwEDEjYhFBQkA19pZABPYmplY3RJZAAYNjRjOTI1NDliYTYyMWVkNWEyOTdiNDkwAAEkAQEBHhQJQXNjZW5kaW5nAANefXQDAQMRQRYUFCQCTggkJAGVAQEBCSgCJAE",
            first: 100
          ) {
            edges { node { id age } }
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
async fn cursor_after_null_one_sort_column_ascending() {
    // This test is based on a cursor that's generated from a field with an id
    // and location of null.

    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          location: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "location": {
                                        "$eq": null
                                    }
                                },
                                {
                                    "_id": {
                                        "$gt": {
                                            "$oid": "64c7a09da6591eea08725b7b"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "location": {
                                "$ne": null
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "location": 1
        },
        "sort": {
            "location": 1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { location: ASC } ],
            after: "ZmllbGRzAG5hbWUACGxvY2F0aW9uAHZhbHVlAAROdWxsAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWMiQDAQMRMiMUFBQDX2lkAE9iamVjdElkABg2NGM3YTA5ZGE2NTkxZWVhMDg3MjViN2IAASQBAQEeFAlBc2NlbmRpbmcAA115awMBAxFBFhQUJAJOCCQkAZEBAQEJKAIkAQ",
            first: 100
          ) {
            edges { node { id location } }
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
async fn cursor_before_null_one_sort_column_ascending() {
    // This test is based on a cursor that's generated from a field with an id
    // and location of null.

    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          location: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$and": [
                        {
                            "location": {
                                "$eq": null
                            }
                        },
                        {
                            "_id": {
                                "$lt": {
                                    "$oid": "64c91fa4ba621ed5a297b48f"
                                }
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "location": 1
        },
        "sort": {
            "location": 1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { location: ASC } ],
            before: "ZmllbGRzAG5hbWUACGxvY2F0aW9uAHZhbHVlAAROdWxsAGRpcmVjdGlvbgAJQXNjZW5kaW5nAAMWMiQDAQMRMiMUFBQDX2lkAE9iamVjdElkABg2NGM5MWZhNGJhNjIxZWQ1YTI5N2I0OGYAASQBAQEeFAlBc2NlbmRpbmcAA115awMBAxFBFhQUJAJOCCQkAZEBAQEJKAIkAQ",
            first: 100
          ) {
            edges { node { id location } }
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
async fn cursor_after_null_one_sort_column_descending() {
    // This test is based on a cursor that's generated from a field with an id
    // and location of null.

    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          location: String
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$and": [
                        {
                            "location": {
                                "$eq": null
                            }
                        },
                        {
                            "_id": {
                                "$gt": {
                                    "$oid": "64c7a09da6591eea08725b7b"
                                }
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "location": 1
        },
        "sort": {
            "location": -1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(
            orderBy: [ { location: DESC } ],
            after: "ZmllbGRzAG5hbWUACGxvY2F0aW9uAHZhbHVlAAROdWxsAGRpcmVjdGlvbgAKRGVzY2VuZGluZwADFzMlAwEDEjMkFBQUA19pZABPYmplY3RJZAAYNjRjN2EwOWRhNjU5MWVlYTA4NzI1YjdiAAEkAQEBHhQJQXNjZW5kaW5nAANeemwDAQMRQRYUFCQCTggkJAGSAQEBCSgCJAE",
            first: 100
          ) {
            edges { node { id location } }
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
async fn cursor_after_two_sort_columns_ascending() {
    // This test is based on a cursor that's generated from a field with an id, name and age.

    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String! @map(name: "real_name")
          age: Int!
        }}
    "#};

    let body = json!({
        "filter": {
            "$and": [
                {},
                {
                    "$or": [
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": null
                                    }
                                },
                                {
                                    "real_name": {
                                        "$eq": "Alice"
                                    }
                                },
                                {
                                    "_id": {
                                        "$gt": {
                                            "$oid": "64cbb12e1ccbe9c84921db52"
                                        }
                                    }
                                }
                            ]
                        },
                        {
                            "$and": [
                                {
                                    "age": {
                                        "$eq": null
                                    }
                                },
                                {
                                    "real_name": {
                                        "$gt": "Alice"
                                    }
                                }
                            ]
                        },
                        {
                            "age": {
                                "$ne": null
                            }
                        }
                    ]
                }
            ]
        },
        "projection": {
            "_id": 1,
            "age": 1,
            "real_name": 1
        },
        "sort": {
            "real_name": 1,
            "age": 1,
            "_id": 1
        },
        "limit": 101,
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query UserCollection {
          userCollection(
            first: 100
            orderBy: [{ name: ASC }, { age: ASC } ],
            after: "ZmllbGRzAG5hbWUAA2FnZQB2YWx1ZQAETnVsbABkaXJlY3Rpb24ACUFzY2VuZGluZwADFi0kAwEDES0jFBQUCXJlYWxfbmFtZQBTdHJpbmcABUFsaWNlAAEPAQEBCxQJQXNjZW5kaW5nAANOZVwDAQMRMhYUFCQDX2lkAE9iamVjdElkABg2NGNiYjEyZTFjY2JlOWM4NDkyMWRiNTIAASQBAQEeFAlBc2NlbmRpbmcAA5WsowMBAxFBFhQUJAOGTwkkJCQBxgEBAQsoAiQB"
          ) {
            edges {
              node {
                id
                age
                name
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

#[tokio::test(flavor = "multi_thread")]
async fn last_no_sort() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {},
        "projection": {
            "_id": 1,
            "name": 1
        },
        "sort": {
            "_id": -1
        },
        "limit": 2
    });

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64c7dc46b73a048947eebd55",
                "name": "Bob"
            },
            {
                "_id": "64cbb12e1ccbe9c84921db52",
                "name": "Alice"
            },
        ]
    })));

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(last: 1) {
            edges { node { name } }
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
                "name": "Bob"
              }
            }
          ]
        }
      }
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn last_with_sort() {
    let config = indoc::formatdoc! {r#"
        type User @model(connector: "{MONGODB_CONNECTOR}", collection: "users") {{
          name: String
        }}
    "#};

    let body = json!({
        "filter": {},
        "projection": {
            "_id": 1,
            "name": 1
        },
        "sort": {
            "_id": -1,
            "name": 1
        },
        "limit": 2
    });

    let mut server = Server::find_many(&config, "users", body).await;
    server.set_response(ResponseTemplate::new(200).set_body_json(json!({
        "documents": [
            {
                "_id": "64cbb12e1ccbe9c84921db52",
                "name": "Alice"
            },
            {
                "_id": "64c7dc46b73a048947eebd55",
                "name": "Bob"
            },
        ]
    })));

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(last: 1, orderBy: [ { id: ASC }, { name: DESC }]) {
            edges { node { name } }
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
                "name": "Alice"
              }
            }
          ]
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, orderBy: [{ id: ASC }, { name: DESC }], first: 100) {
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
            "_id": {
                "$eq": {
                    "$oid": "5ca4bbc7a2dd94ee5816238d"
                }
            }
        },
        "projection": {
            "_id": 1
        },
        "sort": {
            "address.street_name": 1,
            "_id": 1
        },
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } }, orderBy: [{ address: { street: ASC } }], first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { ne: "5ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { gt: "5ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { gte: "5ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { lt: "6ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { lte: "6ca4bbc7a2dd94ee5816238d" } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { in: ["5ca4bbc7a2dd94ee5816238d", "6ca4bbc7a2dd94ee5816238d"] } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { id: { nin: ["5ca4bbc7a2dd94ee5816238d", "6ca4bbc7a2dd94ee5816238d"] } }, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { ALL: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { NONE: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { ANY: [
            { id: { eq: "5ca4bbc7a2dd94ee5816238d" } },
            { id: { eq: "6ca4bbc7a2dd94ee5816238d" } }
          ]}, first: 100) {
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { NOT: { id: { eq: "5ca4bbc7a2dd94ee5816238d" } } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { name: { eq: "Bob" } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: 18 } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: 18.1 } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { age: { eq: true } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { date: { eq: "2022-01-12" } }, first: 100)
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
            "date": { "$eq": { "$date": "2022-01-12T02:33:23.067+00:00" } },
        },
        "projection": {
            "_id": 1
        },
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { date: { eq: "2022-01-12T02:33:23.067+00:00" } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { num: { eq: "1.2345" } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { num: { eq: "9223372036854775807" } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { time: { eq: 1565545664 } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { all: [1, 2, 3] } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { size: 3 } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { elemMatch: { eq: 2 } } }, first: 100)
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
        "limit": 101
    });

    let server = Server::find_many(&config, "users", body).await;

    let request = server.request(indoc::indoc! {r#"
        query {
          userCollection(filter: { data: { elemMatch: { street: { eq: "Wall" } } } }, first: 100)
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
        "limit": 101
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
          }, first: 100)
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
        "limit": 101
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
            },
            first: 100
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

use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, GatewayTester, ResponseExt};
use serde_json::{json, Value};

const SCHEMA: &str = r#"
    extend schema @experimental(partialCaching: true)

    type Query {
        usersAndAccounts: [UserOrAccount!]! @resolver(name: "usersAndAccounts")
        nodes: [Node!]! @resolver(name: "nodes")
        namedNodes: [NamedNode!]! @resolver(name: "namedNodes")
    }

    union UserOrAccount = User | Account

    interface Node {
        id: ID!
    }

    interface NamedNode implements Node {
        id: ID!
        name: String!
    }

    type User implements NamedNode & Node {
        id: ID!
        name: String! @cache(maxAge: 140)
        email: String! @cache(maxAge: 130)
        someConstant: String! @cache(maxAge: 120)
        uncached: String!
    }

    type Account implements Node {
        id: ID!
        email: String! @cache(maxAge: 130)
    }

    type Other implements NamedNode & Node {
        id: ID!
        name: String!
    }
"#;

#[test]
fn union_inline_fragments() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                usersAndAccounts {
                    ... on User {
                        name
                        uncached
                    }
                    ... on Account {
                        email
                    }
                }
            }
        "#;

        let data = gateway.execute(QUERY).await.unwrap().into_data::<Value>();

        insta::assert_json_snapshot!(data, @r###"
        {
          "usersAndAccounts": [
            {
              "__typename": "User",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            {
              "__typename": "User",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            {
              "__typename": "Account",
              "email": "account-one@example.com"
            }
          ]
        }
        "###);
    });
}

#[test]
fn union_fragments() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                usersAndAccounts {
                    ...UserFragment
                    ...AccountFragment
                }
            }

            fragment UserFragment on User {
                name
                uncached
            }

            fragment AccountFragment on Account {
                email
            }
        "#;

        let data = gateway.execute(QUERY).await.unwrap().into_data::<Value>();

        insta::assert_json_snapshot!(data, @r###"
        {
          "usersAndAccounts": [
            {
              "__typename": "User",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            {
              "__typename": "User",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            {
              "__typename": "Account",
              "email": "account-one@example.com"
            }
          ]
        }
        "###);
    });
}

#[test]
fn interface_inline_fragments() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                nodes {
                    ... on Node {
                        id
                        ... on User {
                            name
                            uncached
                        }
                        ... on Account {
                            email
                        }
                    }
                }
                namedNodes {
                    ... on Node {
                        id
                    }
                    ... on User {
                        email
                    }
                }
            }
        "#;

        let data = gateway.execute(QUERY).await.unwrap().into_data::<Value>();

        insta::assert_json_snapshot!(data, @r###"
        {
          "nodes": [
            {
              "__typename": "User",
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            {
              "__typename": "User",
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            {
              "__typename": "Account",
              "id": "one",
              "email": "account-one@example.com"
            }
          ],
          "namedNodes": [
            {
              "__typename": "User",
              "id": "one",
              "email": "user-one@example.com"
            },
            {
              "__typename": "User",
              "id": "two",
              "email": "user-two@example.com"
            },
            {
              "__typename": "Other",
              "id": "bloop"
            }
          ]
        }
        "###);
    });
}

#[test]
fn interface_fragments() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                nodes {
                    ...UserAccountFragment
                }
                namedNodes {
                    ...NodeFragment
                    ...UserFragment
                }
            }

            fragment NodeFragment on Node {
                id
            }

            fragment UserAccountFragment on Node {
                id
                ...UserFragment
                ...AccountFragment
            }

            fragment UserFragment on User {
                name
                uncached
            }

            fragment AccountFragment on Account {
                email
            }
        "#;

        let data = gateway.execute(QUERY).await.unwrap().into_data::<Value>();

        insta::assert_json_snapshot!(data, @r###"
        {
          "nodes": [
            {
              "__typename": "User",
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            {
              "__typename": "User",
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            {
              "__typename": "Account",
              "id": "one",
              "email": "account-one@example.com"
            }
          ],
          "namedNodes": [
            {
              "__typename": "User",
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            {
              "__typename": "User",
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            {
              "__typename": "Other",
              "id": "bloop"
            }
          ]
        }
        "###);
    });
}

#[test]
fn union_inline_fragments_with_defer() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                usersAndAccounts {
                    ... @defer {
                        ... on User @defer {
                            name
                            ... @defer {
                                uncached
                            }
                        }
                        ... on Account @defer {
                            email
                        }
                    }
                }
            }
        "#;

        let data = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(data, @r###"
        [
          {
            "data": {
              "usersAndAccounts": [
                {},
                {},
                {}
              ]
            },
            "hasNext": true
          },
          {
            "data": {},
            "path": [
              "usersAndAccounts",
              0
            ],
            "hasNext": true
          },
          {
            "data": {},
            "path": [
              "usersAndAccounts",
              1
            ],
            "hasNext": true
          },
          {
            "data": {},
            "path": [
              "usersAndAccounts",
              2
            ],
            "hasNext": true
          },
          {
            "data": {
              "name": "User one"
            },
            "path": [
              "usersAndAccounts",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "name": "User two"
            },
            "path": [
              "usersAndAccounts",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "email": "account-one@example.com"
            },
            "path": [
              "usersAndAccounts",
              2
            ],
            "hasNext": true
          },
          {
            "data": {
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            "path": [
              "usersAndAccounts",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            "path": [
              "usersAndAccounts",
              1
            ],
            "hasNext": false
          }
        ]
        "###);
    });
}

#[test]
fn union_fragments_with_defer() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                usersAndAccounts {
                    ...UserFragment @defer
                    ...AccountFragment @defer
                }
            }

            fragment UserFragment on User {
                name
                uncached
            }

            fragment AccountFragment on Account {
                email
            }
        "#;

        let data = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(data, @r###"
        [
          {
            "data": {
              "usersAndAccounts": [
                {},
                {},
                {}
              ]
            },
            "hasNext": true
          },
          {
            "data": {
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            "path": [
              "usersAndAccounts",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            "path": [
              "usersAndAccounts",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "email": "account-one@example.com"
            },
            "path": [
              "usersAndAccounts",
              2
            ],
            "hasNext": false
          }
        ]
        "###);
    });
}

#[test]
fn interface_inline_fragments_with_defer() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                nodes {
                    ... on Node @defer {
                        id
                        ... on User {
                            name
                            uncached
                        }
                        ... on Account {
                            email
                        }
                    }
                }
                namedNodes {
                    ... on Node @defer {
                        id
                    }
                    ... on User {
                        email
                    }
                }
            }
        "#;

        let data = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(data, @r###"
        [
          {
            "data": {
              "nodes": [
                {},
                {},
                {}
              ],
              "namedNodes": [
                {
                  "email": "user-one@example.com"
                },
                {
                  "email": "user-two@example.com"
                },
                {}
              ]
            },
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            "path": [
              "nodes",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            "path": [
              "nodes",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "email": "account-one@example.com"
            },
            "path": [
              "nodes",
              2
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "email": "user-one@example.com"
            },
            "path": [
              "namedNodes",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "two",
              "email": "user-two@example.com"
            },
            "path": [
              "namedNodes",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "bloop"
            },
            "path": [
              "namedNodes",
              2
            ],
            "hasNext": false
          }
        ]
        "###);
    });
}

#[test]
fn interface_fragments_with_defer() {
    runtime().block_on(async {
        let gateway = gateway().await;

        const QUERY: &str = r#"
            query {
                nodes {
                    ...UserAccountFragment @defer
                }
                namedNodes {
                    ...NodeFragment
                    ...UserFragment @defer
                }
            }

            fragment NodeFragment on Node {
                id
            }

            fragment UserAccountFragment on Node {
                id
                ...UserFragment
                ...AccountFragment @defer
            }

            fragment UserFragment on User {
                name
                uncached
            }

            fragment AccountFragment on Account {
                email
            }
        "#;

        let data = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(data, @r###"
        [
          {
            "data": {
              "nodes": [
                {},
                {},
                {}
              ],
              "namedNodes": [
                {
                  "id": "one"
                },
                {
                  "id": "two"
                },
                {
                  "id": "bloop"
                }
              ]
            },
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            "path": [
              "nodes",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            "path": [
              "nodes",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "one"
            },
            "path": [
              "nodes",
              2
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "name": "User one",
              "uncached": "dont cache me bro one"
            },
            "path": [
              "namedNodes",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "two",
              "name": "User two",
              "uncached": "dont cache me bro two"
            },
            "path": [
              "namedNodes",
              1
            ],
            "hasNext": true
          },
          {
            "data": {
              "id": "one",
              "email": "account-one@example.com"
            },
            "path": [
              "nodes",
              2
            ],
            "hasNext": false
          }
        ]
        "###);
    });
}

async fn gateway() -> GatewayTester {
    EngineBuilder::new(SCHEMA)
        .with_custom_resolvers(
            RustUdfs::new()
                .resolver("usersAndAccounts", json!([user("one"), user("two"), account("one"),]))
                .resolver("nodes", json!([user("one"), user("two"), account("one"),]))
                .resolver("namedNodes", json!([user("one"), user("two"), other("bloop")])),
        )
        .gateway_builder()
        .await
        .build()
}

fn user(id: &str) -> serde_json::Value {
    let name = format!("User {id}");
    let email = format!("user-{id}@example.com");
    let constant = format!("blah {id}");
    let uncached = format!("dont cache me bro {id}");

    json!({
        "__typename": "User",
        "id": id,
        "name": name,
        "email": email,
        "someConstant": constant,
        "uncached": uncached
    })
}

fn account(id: &str) -> serde_json::Value {
    let email = format!("account-{id}@example.com");

    json!({
        "__typename": "Account",
        "id": id,
        "email": email
    })
}

fn other(id: &str) -> serde_json::Value {
    let name = format!("Other {id}");

    json!({
        "__typename": "Other",
        "id": id,
        "name": name,
    })
}

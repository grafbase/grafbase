//! Tests of the join directive

use graphql_mocks::{ErrorSchema, FakeGithubSchema, MockGraphQlServer};
use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::{json, Value};

#[test]
fn join_on_basic_type() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                greetPerson(name: String): String! @resolver(name: "greetPerson")
                user: User! @resolver(name: "user")
            }

            type User {
                id: ID!
                name: String!
                greeting: String! @join(select: "greetPerson(name: $name)")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("user", UdfResponse::Success(json!({"id": "123", "name": "Bob"})))
                    .resolver("greetPerson", |input: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(
                            format!("Hello {}", input.arguments["name"].as_str().unwrap(),).into(),
                        ))
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute("{ user { greeting } }")
                .await
                .into_data::<Value>(),
                @r###"
        {
          "user": {
            "greeting": "Hello Bob"
          }
        }
        "###
        );
    });
}

#[test]
fn join_on_connector_type() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "gothub",
                    namespace: false,
                    url: "http://127.0.0.1:{port}",
                )

            extend type Query {{
                describeIssue(name: String): String! @resolver(name: "describeIssue")
            }}

            extend type Issue {{
                description: String! @join(select: "describeIssue(name: $title)")
            }}
            "#
        );

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new().resolver("describeIssue", |input: CustomResolverRequestPayload| {
                    Ok(UdfResponse::Success(
                        format!("Oh no {}", input.arguments["name"].as_str().unwrap(),).into(),
                    ))
                }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    pullRequestOrIssue(id: "3") {
                        ... on Issue {
                            title
                            description
                        }
                    }
                }
                "#)
                .await
                .into_data::<Value>(),
                @r###"
        {
          "pullRequestOrIssue": {
            "title": "Everythings fine",
            "description": "Oh no Everythings fine"
          }
        }
        "###
        );
    });
}

#[test]
fn multiple_joins_on_graphql_connector() {
    // Tests the case where a GraphQL connector join appears twice in a response, which leads to a
    // single additional GraphQL request.  The naive approach to serializing this would build an
    // invalid GraphQL query - this test makes sure that it works.
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "gothub",
                    namespace: false,
                    url: "http://127.0.0.1:{port}",
                )

            extend type PullRequest {{
                oohRecursion: PullRequest @join(select: "pullRequest(id: $id)")
            }}
            "#
        );

        let engine = EngineBuilder::new(schema).build().await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    pullRequestsAndIssues(filter: {search: ""}) {
                        ... on PullRequest {
                            id
                            oohRecursion {
                                id
                                title
                            }
                        }
                    }
                }
                "#)
                .await
                .into_data::<Value>(),
                @r###"
        {
          "pullRequestsAndIssues": [
            {
              "id": "1",
              "oohRecursion": {
                "id": "1",
                "title": "Creating the thing"
              }
            },
            {
              "id": "2",
              "oohRecursion": {
                "id": "2",
                "title": "Some bot PR"
              }
            },
            {}
          ]
        }
        "###
        );
    });
}

#[test]
fn multiple_joins_on_namespaced_graphql_connector() {
    // Tests the case where a GraphQL connector join appears twice in a response, which leads to a
    // single additional GraphQL request.  The naive approach to serializing this would build an
    // invalid GraphQL query - this test makes sure that it works when the graphql connector
    // is namespaced.
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "gothub",
                    namespace: true
                    url: "http://127.0.0.1:{port}",
                )

            extend type GothubPullRequest {{
                oohRecursion: GothubPullRequest @join(select: "gothub {{ pullRequest(id: $id) }}")
            }}
            "#
        );

        let engine = EngineBuilder::new(schema).build().await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    gothub {
                        pullRequestsAndIssues(filter: {search: ""}) {
                            ... on GothubPullRequest {
                                id
                                oohRecursion {
                                    id
                                    title
                                }
                            }
                        }
                    }
                }
                "#)
                .await
                .into_data::<Value>(),
                @r###"
        {
          "gothub": {
            "pullRequestsAndIssues": [
              {
                "id": "1",
                "oohRecursion": {
                  "id": "1",
                  "title": "Creating the thing"
                }
              },
              {
                "id": "2",
                "oohRecursion": {
                  "id": "2",
                  "title": "Some bot PR"
                }
              },
              {}
            ]
          }
        }
        "###
        );
    });
}

#[test]
fn join_with_an_enum_argument() {
    // Tests the case where we're providing an enum argument to a joined field.
    // ResolveValues use JSON which represent enums as a String, but we need to render
    // those as enums in the query we build up.  This makes sure that works properly.
    //
    // Though it seems to be a terrible test because AsyncGraphql doesn't give a shit
    // if you give it a string where it expects an enum :|
    runtime().block_on(async {
        let mut graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "gothub",
                    namespace: false,
                    url: "http://127.0.0.1:{port}",
                )

            extend type PullRequest {{
                statusText: String! @join(select: "statusString(status: $status)")
            }}
            "#
        );

        let engine = EngineBuilder::new(schema).build().await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    pullRequest(id: "1") {
                        id
                        status
                        statusText
                    }
                }
                "#)
                .await
                .into_data::<Value>(),
                @r###"
        {
          "pullRequest": {
            "id": "1",
            "status": "OPEN",
            "statusText": "boo its closed"
          }
        }
        "###
        );

        // AsyncGraphQL doesn't seem to care if you give it a String in Enum position.
        // So lets snapshot the request just to be sure this doesn't regress.
        let requests = graphql_mock.drain_requests().await.collect::<Vec<_>>();
        assert_eq!(requests.len(), 3, "Unexpected requests: {requests:?}");
        let request = requests.last().unwrap();

        insta::assert_snapshot!(request.query, @r###"
        query {
        	field_0: statusString(status: OPEN)
        }
        "###);
    });
}

#[test]
fn nested_joins() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                greetings(name: String!): Greetings @resolver(name: "greetings")
                user: User! @resolver(name: "user")
            }

            type User {
                id: ID!
                name: String!
                greeting: String! @join(
                    select: "greetings(name: $name) { forTimeOfDay(id: $id, timeOfDay: \"morning\") }"
                )
            }

            type Greetings {
                forTimeOfDay(id: String!, timeOfDay: String!): String! @resolver(name: "timeOfDayGreeting")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("user", UdfResponse::Success(json!({"id": "123", "name": "Bob"})))
                    .resolver("greetings", |input: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(json!({"name": input.arguments["name"]})))
                    })
                    .resolver("timeOfDayGreeting", |input: CustomResolverRequestPayload| {
                        let time_of_day = input.arguments["timeOfDay"].as_str().unwrap();
                        let id = input.arguments["id"].as_str().unwrap();
                        let name = input.parent.as_ref().unwrap()["name"].as_str().unwrap();

                        Ok(UdfResponse::Success(
                            format!("Good {time_of_day} {name} your ID is {id}").into(),
                        ))
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute("{ user { greeting } }")
                .await
                .into_data::<Value>(),
                @r###"
        {
          "user": {
            "greeting": "Good morning Bob your ID is 123"
          }
        }
        "###
        );
    });
}

#[test]
fn joins_with_downstream_errors() {
    // Tests the case where we're joining onto a GraphQL connector, but that GraphQL connector
    // returns errors
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(ErrorSchema::default()).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "errors",
                    namespace: true,
                    url: "http://127.0.0.1:{port}",
                )

            type JoinContainer {{
                brokenObjectList: [ErrorsBrokenObject] @join(select: "errors {{ brokenObjectList(error: \"objectError\") }}")
            }}

            extend type Query {{
                joins: [JoinContainer]! @resolver(name: "joinContainer")
            }}
            "#
        );

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver("joinContainer", UdfResponse::Success(json!([{}, {}]))))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    joins {
                        brokenObjectList {
                            brokenField
                        }
                    }
                }
                "#)
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "joins": [
              {
                "brokenObjectList": [
                  null,
                  null
                ]
              },
              {
                "brokenObjectList": [
                  null,
                  null
                ]
              }
            ]
          },
          "errors": [
            {
              "message": "objectError",
              "path": [
                "joins",
                0,
                "brokenObjectList",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectError",
              "path": [
                "joins",
                0,
                "brokenObjectList",
                "brokenObjectList",
                1,
                "brokenField"
              ]
            },
            {
              "message": "objectError",
              "path": [
                "joins",
                1,
                "brokenObjectList",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectError",
              "path": [
                "joins",
                1,
                "brokenObjectList",
                "brokenObjectList",
                1,
                "brokenField"
              ]
            }
          ]
        }
        "###
        );
    });
}

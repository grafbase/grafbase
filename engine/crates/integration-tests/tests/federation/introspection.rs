use cynic::{http::ReqwestExt, QueryBuilder};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SpecificationVersion};
use engine_v2::Engine;
use integration_tests::{
    federation::EngineV2Ext,
    mocks::graphql::{EchoSchema, FakeGithubSchema},
    runtime, MockGraphQlServer,
};

const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("./graphql/introspection.graphql");

#[test]
fn can_run_pathfinder_introspection_query() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine.execute(PATHFINDER_INTROSPECTION_QUERY).await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
    }

    scalar CustomRepoId

    type Header {
      name: String!
      value: String!
    }

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      favoriteRepository: CustomRepoId!
      headers: [Header!]!
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
    }

    type User {
      email: String!
      name: String!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn can_run_2018_introspection_query() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine
            .execute(IntrospectionQuery::with_capabilities(
                SpecificationVersion::June2018.capabilities(),
            ))
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
    }

    scalar CustomRepoId

    type Header {
      name: String!
      value: String!
    }

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      favoriteRepository: CustomRepoId!
      headers: [Header!]!
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
    }

    type User {
      email: String!
      name: String!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn can_run_2021_introspection_query() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine
            .execute(IntrospectionQuery::with_capabilities(
                SpecificationVersion::October2021.capabilities(),
            ))
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
    }

    scalar CustomRepoId

    type Header {
      name: String!
      value: String!
    }

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      favoriteRepository: CustomRepoId!
      headers: [Header!]!
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
    }

    type User {
      email: String!
      name: String!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn can_run_capability_introspection_query() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine.execute(CapabilitiesQuery::build(())).await
    });
    assert!(response.errors().is_empty(), "{response}");

    let response = serde_json::from_value::<CapabilitiesQuery>(response.into_data()).expect("valid response");

    assert_eq!(
        response.capabilities().version_supported(),
        SpecificationVersion::October2021
    );
}

#[test]
#[ignore]
#[allow(clippy::panic)]
fn introspection_output_matches_source() {
    use reqwest::Client;
    let (response, _upstream_sdl) = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        let response = engine.execute(IntrospectionQuery::build(())).await;

        let upstream_sdl = Client::new()
            .post(format!("http://localhost:{}", github_mock.port()))
            .run_graphql(IntrospectionQuery::build(()))
            .await
            .expect("request to work")
            .data
            .expect("data to be present")
            .into_schema()
            .expect("valid schema")
            .to_sdl();

        (response, upstream_sdl)
    });
    assert!(response.errors().is_empty(), "{response}");

    let _engine_sdl = introspection_to_sdl(response.into_data());

    panic!("How to compare efficiently to DSL? They don't have the same ordering of fields or types.");
}

#[test]
fn can_introsect_when_multiple_subgraphs() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let echo_mock = MockGraphQlServer::new(EchoSchema::default()).await;

        let engine = Engine::build()
            .with_schema("github", &github_mock)
            .await
            .with_schema("echo", &echo_mock)
            .await
            .finish();

        engine.execute(IntrospectionQuery::build(())).await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
    }

    scalar CustomRepoId

    type Header {
      name: String!
      value: String!
    }

    input InputObj {
      string: String
      int: Int
      float: Float
      id: ID
      annoyinglyOptionalStrings: [[String!]]!
      recursiveObject: InputObj
      recursiveObjectList: [InputObj!]!
    }

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    scalar JSON

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      favoriteRepository: CustomRepoId!
      float(input: Float!): Float!
      headers: [Header!]!
      id(input: ID!): ID!
      inputObject(input: InputObj!): JSON!
      int(input: Int!): Int!
      listOfListOfStrings(input: [[String!]!]!): [[String!]!]!
      listOfStrings(input: [String!]!): [String!]!
      optionalListOfOptionalStrings(input: [[String!]]!): [[String!]]!
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
      string(input: String!): String!
    }

    type User {
      email: String!
      name: String!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn supports_the_type_field() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish();

        engine
            .execute(
                r#"
                    query {
                        __type(name: "PullRequest") {
                            kind
                            name
                            description
                            fields(includeDeprecated: true) {
                                name
                            }
                            interfaces {
                                name
                            }
                            possibleTypes {
                                name
                            }
                            enumValues {
                                name
                            }
                            inputFields {
                                name
                            }
                            ofType {
                                kind
                                name
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": {
          "kind": "OBJECT",
          "name": "PullRequest",
          "description": null,
          "fields": [
            {
              "name": "author"
            },
            {
              "name": "checks"
            },
            {
              "name": "title"
            }
          ],
          "interfaces": [
            {
              "name": "PullRequestOrIssue"
            }
          ],
          "possibleTypes": null,
          "enumValues": null,
          "inputFields": null,
          "ofType": null
        }
      }
    }
    "###);
}

#[test]
fn type_field_returns_null_on_missing_type() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish();

        engine
            .execute(
                r#"
                    query {
                        __type(name: "Boom") {
                            kind
                            name
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": null
      }
    }
    "###);
}

#[test]
fn supports_recursing_through_types() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish();

        engine
            .execute(
                r#"
                    query {
                        __type(name: "PullRequestOrIssue") {
                            possibleTypes {
                                name
                                interfaces {
                                    name
                                    possibleTypes {
                                        name
                                        interfaces {
                                            name
                                            possibleTypes {
                                                name
                                                interfaces {
                                                    name
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": {
          "possibleTypes": [
            {
              "name": "Issue",
              "interfaces": [
                {
                  "name": "PullRequestOrIssue",
                  "possibleTypes": [
                    {
                      "name": "Issue",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "PullRequest",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": "PullRequest",
              "interfaces": [
                {
                  "name": "PullRequestOrIssue",
                  "possibleTypes": [
                    {
                      "name": "Issue",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "PullRequest",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      }
    }
    "###);
}

#[test]
fn rejects_bogus_introspection_queries() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("github", &github_mock).await.finish();

        engine
            .execute(
                r#"
                    query {
                        __type(name: "PullRequestOrIssue") {
                            possibleTypes {
                                blarg
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "locations": [
            {
              "line": 5,
              "column": 33
            }
          ],
          "message": "__Type does not have a field named 'blarg'"
        }
      ]
    }
    "###);
}

#[allow(clippy::panic)]
fn introspection_to_sdl(data: serde_json::Value) -> String {
    serde_json::from_value::<IntrospectionQuery>(data)
        .expect("valid response")
        .into_schema()
        .expect("valid schema")
        .to_sdl()
}

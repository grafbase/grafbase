use cynic::{http::ReqwestExt, QueryBuilder};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SpecificationVersion};
use engine_v2::Engine;
use integration_tests::{
    federation::EngineV2Ext,
    mocks::graphql::{EchoSchema, FakeGithubSchema},
    runtime, MockGraphQlServer,
};

#[test]
#[ignore]
fn can_run_2018_introspection_query() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(IntrospectionQuery::with_capabilities(
                    SpecificationVersion::June2018.capabilities(),
                ))
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(introspection_to_sdl(response), @"");
}

#[test]
#[ignore]
fn can_run_2021_introspection_query() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(IntrospectionQuery::with_capabilities(
                    SpecificationVersion::October2021.capabilities(),
                ))
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(introspection_to_sdl(response), @"");
}

#[test]
#[ignore]
fn can_run_capability_introspection_query() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine.execute(CapabilitiesQuery::build(())).await
        })
        .unwrap();

    let response = serde_json::from_value::<CapabilitiesQuery>(response).expect("valid response");

    assert_eq!(
        response.capabilities().version_supported(),
        SpecificationVersion::October2021
    );
}

#[test]
#[ignore]
fn introspection_output_matches_source() {
    use reqwest::Client;
    let (engine_response, downstream_sdl) = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        let engine_response = engine.execute(IntrospectionQuery::build(())).await.unwrap();

        let downstream_sdl = Client::new()
            .post(format!("http://localhost:{}", github_mock.port()))
            .run_graphql(IntrospectionQuery::build(()))
            .await
            .expect("request to work")
            .data
            .expect("data to be present")
            .into_schema()
            .expect("valid schema")
            .to_sdl();

        (engine_response, downstream_sdl)
    });

    let engine_sdl = introspection_to_sdl(engine_response);

    similar_asserts::assert_eq!(engine_sdl, downstream_sdl);
}

#[test]
#[ignore]
fn can_introsect_when_multiple_subgraphs() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;
            let echo_mock = MockGraphQlServer::new(EchoSchema::default()).await;

            let engine = Engine::build()
                .with_schema("github", &github_mock)
                .await
                .with_schema("echo", &echo_mock)
                .await
                .finish();

            engine.execute(IntrospectionQuery::build(())).await
        })
        .unwrap();

    insta::assert_json_snapshot!(introspection_to_sdl(response), @"");
}

#[test]
#[ignore]
fn supports_the_type_field() {
    let response = runtime()
        .block_on(async move {
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
                            interfaces
                            possibleTypes
                            enumValues
                            inputFields {
                                blah
                            }
                            ofType {
                                blah
                            }
                        }
                    }
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn type_field_returns_null_on_missing_type() {
    let response = runtime()
        .block_on(async move {
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
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn supports_recursing_through_types() {
    let response = runtime()
        .block_on(async move {
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
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn rejects_bogus_introspection_queries() {
    let response = runtime()
        .block_on(async move {
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
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

fn introspection_to_sdl(response: serde_json::Value) -> String {
    serde_json::from_value::<IntrospectionQuery>(response)
        .expect("valid response")
        .into_schema()
        .expect("valid schema")
        .to_sdl()
}

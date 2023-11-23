//! Tests of the join directive

use integration_tests::{
    mocks::graphql::FakeGithubSchema, runtime, udfs::RustUdfs, EngineBuilder, MockGraphQlServer, ResponseExt,
};
use runtime::udf::{CustomResolverRequestPayload, CustomResolverResponse};
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
                    .resolver(
                        "user",
                        CustomResolverResponse::Success(json!({"id": "123", "name": "Bob"})),
                    )
                    .resolver("greetPerson", |input: CustomResolverRequestPayload| {
                        Ok(CustomResolverResponse::Success(
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
                    Ok(CustomResolverResponse::Success(
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
            "description": "Oh no Everythings fine",
            "title": "Everythings fine"
          }
        }
        "###
        );
    });
}

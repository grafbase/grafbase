//! This file tests that the requires directive works correctly when we're entirely
//! internal to federation

use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::{json, Value};

#[test]
fn test_requires_when_field_present() {
    // Tests that `@requires` works correctly when the required fields are already
    // provided by a parent resolver
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                user: User! @resolver(name: "user")
            }

            type User {
                id: ID!
                name: String!
                greeting: String! @requires(fields: "id name") @resolver(name: "greeting")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("user", UdfResponse::Success(json!({"id": "123", "name": "Bob"})))
                    .resolver("greeting", |input: CustomResolverRequestPayload| {
                        let parent = input.parent.unwrap();
                        Ok(UdfResponse::Success(
                            format!(
                                "Hello {} (ID: {})",
                                parent["name"].as_str().unwrap(),
                                parent["id"].as_str().unwrap()
                            )
                            .into(),
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
            "greeting": "Hello Bob (ID: 123)"
          }
        }
        "###
        );
    });
}

#[test]
fn field_present_but_requires_rename() {
    // Tests that `@requires` works correctly when the required fields are present
    // in parent_resolve_value but require a rename step to run first
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                user: User! @resolver(name: "user")
            }

            type User {
                id: ID!
                name: String! @map(name: "Name_Of_User")
                greeting: String! @requires(fields: "id name") @resolver(name: "greeting")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver(
                        "user",
                        UdfResponse::Success(json!({"id": "123", "Name_Of_User": "Bob"})),
                    )
                    .resolver("greeting", |input: CustomResolverRequestPayload| {
                        let parent = input.parent.unwrap();
                        Ok(UdfResponse::Success(
                            format!(
                                "Hello {} (ID: {})",
                                parent["name"].as_str().unwrap(),
                                parent["id"].as_str().unwrap()
                            )
                            .into(),
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
            "greeting": "Hello Bob (ID: 123)"
          }
        }
        "###
        );
    });
}

#[test]
fn field_not_present_requires_other_resolver() {
    // Tests that `@requires` works correctly when the required fields need another
    // resolver to run first (and that resolver _also_ has some requires)
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                user: User! @resolver(name: "user")
            }

            type User {
                id: ID! @map(name: "ID")
                name: String! @requires(fields: "id") @resolver(name: "username")
                greeting: String! @requires(fields: "name") @resolver(name: "greeting")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("user", json!({"ID": "123"}))
                    .resolver("username", |input: CustomResolverRequestPayload| {
                        let parent = input.parent.unwrap();
                        Ok(UdfResponse::Success(
                            format!("User {}", parent["id"].as_str().unwrap()).into(),
                        ))
                    })
                    .resolver("greeting", |input: CustomResolverRequestPayload| {
                        let parent = input.parent.unwrap();
                        Ok(UdfResponse::Success(
                            format!("Hello {}", parent["name"].as_str().unwrap(),).into(),
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
            "greeting": "Hello User 123"
          }
        }
        "###
        );
    });
}

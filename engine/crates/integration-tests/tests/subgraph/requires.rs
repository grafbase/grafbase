use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::{json, Value};

const USER_SCHEMA: &str = r#"
    extend schema @federation(version: "2.3")

    type User @key(fields: "id") {
        id: ID!
        name: String!
        greeting: String! @requires(fields: "name") @resolver(name: "greeting")
    }
"#;

#[test]
fn test_federation_with_required_field() {
    // Tests that `@requires` works correctly when the required field is correctly provided by the
    // parent resolver.
    runtime().block_on(async {
        let engine = EngineBuilder::new(USER_SCHEMA)
            .with_custom_resolvers(
                RustUdfs::new().resolver("greeting", |input: CustomResolverRequestPayload| {
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
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on User {
                                greeting
                            }
                        }
                    }
                ",
                )
                .variables(json!({"repr": {
                    "__typename": "User",
                    "id": "123",
                    "name": "Bob"
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "User",
              "greeting": "Hello Bob (ID: 123)"
            }
          ]
        }
        "###
        );
    });
}

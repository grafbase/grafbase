//! Tests of union types

use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::{json, Value};

const QUERY: &str = r#"
query($name: String!) {
    fetchAPet(name: $name) {
        __typename
        name
        ... on Cat {
            claws
        }
        ... on Dog {
            friend
        }
    }
}
"#;

#[test]
fn interface_type_fetching_dog() {
    runtime().block_on(async {
        let engine = setup_engine().await;

        insta::assert_json_snapshot!(
            engine
                .execute(QUERY)
                .variables(json!({"name": "Fido"}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "fetchAPet": {
            "__typename": "Dog",
            "name": "Fido",
            "friend": "EVERYONE_IS_FRIEND"
          }
        }
        "###
        );
    });
}

#[test]
fn interface_type_fetching_cat() {
    runtime().block_on(async {
        let engine = setup_engine().await;

        insta::assert_json_snapshot!(
            engine
                .execute(QUERY)
                .variables(json!({"name": "Luna"}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "fetchAPet": {
            "__typename": "Cat",
            "name": "Luna",
            "claws": "VerySharp"
          }
        }
        "###
        );
    });
}

#[test]
fn interface_type_fetching_null() {
    runtime().block_on(async {
        let engine = setup_engine().await;

        insta::assert_json_snapshot!(
            engine
                .execute(QUERY)
                .variables(json!({"name": "Fuzz"}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "fetchAPet": null
        }
        "###
        );
    });
}

async fn setup_engine() -> Engine {
    let schema = r#"
        extend type Query {
            fetchAPet(name: String!): Pet @resolver(name: "fetchPet")
        }

        interface Pet {
            name: String!
        }

        type Cat implements Pet {
            name: String!
            claws: Sharpness!
        }

        enum Sharpness {
            VerySharp
        }

        type Dog implements Pet {
            name: String!
            friend: Friendliness
        }

        enum Friendliness {
            EveryoneIsFriend
        }
    "#;

    EngineBuilder::new(schema)
        .with_custom_resolvers(
            RustUdfs::new().resolver("fetchPet", |input: CustomResolverRequestPayload| {
                Ok(UdfResponse::Success(if input.arguments["name"] == json!("Fido") {
                    json!({
                        "__typename": "Dog",
                        "name": "Fido",
                        "friend": "EVERYONE_IS_FRIEND"
                    })
                } else if input.arguments["name"] == json!("Luna") {
                    json!({
                        "__typename": "Cat",
                        "name": "Luna",
                        "claws": "VerySharp"
                    })
                } else {
                    json!(null)
                }))
            }),
        )
        .build()
        .await
}

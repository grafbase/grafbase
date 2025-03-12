mod arguments;
mod field;
mod type_condition;
mod validation;

use crate::federation::extensions::resolver::validation::EchoExt;
use engine::Engine;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSchemaBuilder};
use integration_tests::{
    federation::{EngineExt, GraphqlResponse},
    runtime,
};
use serde_json::json;

fn run_with_field_set(subgraph: DynamicSchemaBuilder, field_set: &str) -> Result<GraphqlResponse, String> {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph(subgraph.into_subgraph("a"))
            .with_subgraph_sdl(
                "b",
                &format!(
                    r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type User {{
                    echo: JSON @external @echo(fields: "{field_set}")
                }}
                "#
                ),
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet"])

                directive @echo(fields: FieldSet!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await?
            .post(r#"query { user { echo } }"#)
            .await;
        Ok(response)
    })
}

fn graphql_subgraph() -> DynamicSchemaBuilder {
    DynamicSchema::builder(
        r#"
        type Query {
            user: User
        }

        type User {
            id: ID!
            name: String!
            age: Int!
            address: Address
            friends: [User!]
            pets: [Pet!]!
        }

        type Address {
            street: String!
            city: String!
            country: String!
        }

        union Pet = Dog | Cat

        type Dog {
            id: ID!
            name: String!
        }

        type Cat {
            id: ID!
            name: String!
        }

        "#,
    )
    .with_resolver(
        "Query",
        "user",
        json!({
            "id": "1",
            "name": "Peter",
            "age": 3,
            "address": {"street": "123 Main St", "city": "Springfield", "country": "USA"},
            "friends": [
                {"id": "2", "name": "Alice", "age": 3},
                {"id": "3", "name": "Bob", "age": 4}
            ],
            "pets": [
                { "__typename": "Dog", "id": "1", "name": "Fido" },
                { "__typename": "Cat", "id": "2", "name": "Whiskers" },
            ],

        }),
    )
}

#[test]
fn basic_field_set() {
    let response = run_with_field_set(graphql_subgraph(), "id").unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "id": "1"
              }
            }
          }
        }
      }
    }
    "#);
}

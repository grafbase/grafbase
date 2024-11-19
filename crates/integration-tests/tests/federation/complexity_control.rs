use async_graphql::Object;
use engine::Engine;
use graphql_mocks::{DynamicSchema, MockGraphQlServer, Subgraph};
use integration_tests::{federation::EngineExt, runtime};

const LIMIT_CONFIG: &str = r#"
[complexity_control]
mode = "enforce"
limit = 100
"#;

#[test]
fn test_uncomplex_query() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        engine.post("query { field }").await
    });

    similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"field": null}}));
}

#[test]
fn test_complex_query_while_off() {}

#[test]
fn test_complex_query_with_enforce() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        engine.post("query { expensiveField }").await
    });

    insta::assert_json_snapshot!(response.body, @"")
}

#[test]
fn test_complex_query_with_monitor() {}

#[derive(Default)]
pub struct ComplexitySchema;

impl Subgraph for ComplexitySchema {
    fn name(&self) -> String {
        "complexity".into()
    }

    async fn start(self) -> MockGraphQlServer {
        let schema = DynamicSchema::builder(
            r#"
            type Query {
                field: String

                expensiveField: String @cost(weight: 100)

                sizedListField: [Item] @listSize(assumedListSize: 50)

                cursorListField(first: Int, last: Int): Connection
            }

            # TODO: add arguments that cost
            # TODO: add inpput object fields that cost

            type Connection {
                items: [Item]
            }

            type Item {
                blah: String!
            }
            "#,
        )
        .finish();

        MockGraphQlServer::new(schema).await
    }
}

#[Object]
impl ComplexitySchema {
    async fn string(&self, input: String) -> String {
        input
    }
}

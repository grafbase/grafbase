use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use integration_tests::{
    federation::{FieldResolverExt, FieldResolverTestExtension, Gateway, json_data},
    runtime,
};
use runtime::extension::Data;

#[derive(Default, Clone)]
pub struct GreetExt;

impl GreetExt {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> FieldResolverExt {
        FieldResolverExt::new(Self).with_name("greet").with_sdl(
            r#"
            directive @greet on FIELD_DEFINITION
            "#,
        )
    }
}

#[async_trait::async_trait]
impl FieldResolverTestExtension for GreetExt {
    async fn resolve_field(
        &self,
        _directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Ok(vec![Ok(json_data("Hi!")); inputs.len()])
    }
}

#[test]
fn simple_resolver_from_federated_sdl() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_federated_sdl(
                r#"
                enum extension__Link {
                    REST @extension__link(url: "greet-1.0.0")
                }

                enum join__Graph {
                    A @join__graph(name: "a")
                }

                extend type Query {
                    greeting(name: String): String @extension__directive(graph: A, extension: REST, name: "greet")
                }
                "#,
            )
            .with_extension(GreetExt::new())
            .build()
            .await;

        engine.post("query { greeting }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": "Hi!"
      }
    }
    "#);
}

#[test]
fn simple_resolver_from_subgraph_sdl() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "greet-1.0.0", import: ["@greet"])

                type Query {
                    greeting(name: String): String @greet
                }
                "#,
            )
            .with_extension(GreetExt::new())
            .build()
            .await;

        engine.post("query { greeting }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": "Hi!"
      }
    }
    "#);
}

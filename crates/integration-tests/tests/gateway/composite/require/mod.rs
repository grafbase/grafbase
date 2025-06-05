use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::Subgraph;
use extension_catalog::{ExtensionId, Id};
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{
    gateway::{AnyExtension, Gateway, ResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data};
use serde_json::json;

#[derive(Clone)]
pub struct Resolve {
    resolve: Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, GraphqlError> + Send + Sync>,
}

impl Resolve {
    pub fn with<F>(f: F) -> Self
    where
        F: Fn(serde_json::Value) -> Result<serde_json::Value, GraphqlError> + Send + Sync + 'static,
    {
        Self { resolve: Arc::new(f) }
    }
}

impl AnyExtension for Resolve {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "resolver".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType { directives: None }),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        });
        state.test.selection_set_resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for Resolve {
    async fn resolve(
        &self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        mut arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError> {
        assert!(arguments.len() == 1);
        let (_, arg) = arguments.pop().unwrap();
        (self.resolve)(arg).map(|value| Data::Json(serde_json::to_vec(&value).unwrap().into()))
    }
}

fn gql_product() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products: [Product!]!
        }

        type Product @key(fields: "id") {
            id: ID!
        }
        "#,
    )
    .with_resolver("Query", "products", json!([{"id": "1"}, {"id": "2"}]))
    .into_subgraph("gql")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key"])
                    @init

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: ID! @require(field: "id")): JSON @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(Ok))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "id": "1"
                }
              },
              {
                "id": "2",
                "dummy": {
                  "id": "2"
                }
              }
            ]
          }
        }
        "#);
    })
}

#[test]
fn basic_batch() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key"])
                    @init

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: [ID!]! @require(field: "[id]")): JSON @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(|args| Ok(args["id"].clone())))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": "1"
              },
              {
                "id": "2",
                "dummy": "2"
              }
            ]
          }
        }
        "#);
    })
}

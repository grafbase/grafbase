mod is;
mod key;
mod shape;

use integration_tests::{gateway::Gateway, runtime};

use crate::gateway::extensions::resolver::ResolverExt;

use std::sync::Arc;

use engine_schema::ExtensionDirective;
use extension_catalog::Id;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::gateway::{AnyExtension, ResolverTestExtension, TestManifest};
use runtime::extension::{ArgumentsId, Data, Response};
use serde_json::json;

#[derive(Clone)]
struct EchoLookup {
    batch: bool,
    namespace: Option<&'static str>,
}

impl EchoLookup {
    pub fn single() -> Self {
        Self {
            batch: false,
            namespace: None,
        }
    }

    pub fn batch() -> Self {
        Self {
            batch: true,
            namespace: None,
        }
    }

    pub fn namespaced(self, namespace: &'static str) -> Self {
        Self {
            namespace: Some(namespace),
            ..self
        }
    }
}

impl AnyExtension for EchoLookup {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "echo".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(Default::default()),
            sdl: Some(r#"directive @echo on FIELD_DEFINITION"#),
        });
        state.test.resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for EchoLookup {
    async fn resolve(
        &self,
        _directive: ExtensionDirective<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        mut arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        assert!(arguments.len() == 1);
        let (_, arg) = arguments.pop().unwrap();
        if self.batch {
            if let Some(namespace) = self.namespace {
                Response::data(Data::Json(
                    serde_json::to_vec(&serde_json::json!({namespace: [{"args": arg}]}))
                        .unwrap()
                        .into(),
                ))
            } else {
                Response::data(Data::Json(
                    serde_json::to_vec(&serde_json::json!([{"args": arg}])).unwrap().into(),
                ))
            }
        } else if let Some(namespace) = self.namespace {
            Response::data(Data::Json(
                serde_json::to_vec(&serde_json::json!({namespace: {"args": arg}}))
                    .unwrap()
                    .into(),
            ))
        } else {
            Response::data(Data::Json(
                serde_json::to_vec(&serde_json::json!({"args": arg})).unwrap().into(),
            ))
        }
    }
}

fn gql_id() -> DynamicSubgraph {
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
    .with_resolver("Query", "products", json!([{"id": "1"}]))
    .into_subgraph("gql")
}

fn gql2_name() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products2: [Product!]!
        }

        type Product @key(fields: "name") {
            name: String!
        }
        "#,
    )
    .with_resolver("Query", "products2", json!([{"name": "name1"}]))
    .into_subgraph("gql2")
}

fn gql_ab() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products: [Product!]!
        }
        type Product @key(fields: "a b") {
            a: ID!
            b: ID!
        }
        "#,
    )
    .with_resolver("Query", "products", json!([{"a": "A1", "b": "B1"}]))
    .into_subgraph("gql")
}

fn gql_ab_id_int() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products: [Product!]!
        }
        type Product @key(fields: "a b") {
            a: ID!
            b: Int!
        }
        "#,
    )
    .with_resolver("Query", "products", json!([{"a": "A1", "b": 1}]))
    .into_subgraph("gql")
}

fn gql_nested() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products: [Product!]!
        }

        type Product @key(fields: "nested { id }") {
            nested: Nested!
        }

        type Nested @shareable {
            id: ID!
        }
        "#,
    )
    .with_resolver("Query", "products", json!([{"nested": { "id": "1"} }]))
    .into_subgraph("gql")
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
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
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])


                type Query {
                    productBatch(ids: [ID!]!): [Product!]! @lookup @resolve
                }

                type Product @key(fields: "id") {
                    id: ID!
                    code: String!
                }
                "#,
            )
            .with_extension(ResolverExt::json(json!([{"code": "C1"}, {"code": "C2"}])))
            .build()
            .await;

        let response = engine.post("query { products { id code } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "code": "C1"
              },
              {
                "id": "2",
                "code": "C2"
              }
            ]
          }
        }
        "#);
    })
}

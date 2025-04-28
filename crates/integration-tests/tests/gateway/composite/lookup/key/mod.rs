mod batch;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::Subgraph;
use extension_catalog::{ExtensionId, Id};
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::gateway::{AnyExtension, SelectionSetResolverTestExtension, TestManifest};
use runtime::extension::{ArgumentsId, Data};
use serde_json::json;

#[derive(Clone)]
struct EchoArgs;

impl AnyExtension for EchoArgs {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "static".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::SelectionSetResolver(Default::default()),
            sdl: Some(r#"directive @init on SCHEMA"#),
        });
        state.test.selection_set_resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn SelectionSetResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl SelectionSetResolverTestExtension for EchoArgs {
    async fn resolve_field(
        &self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        mut arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError> {
        println!("{arguments:#?}");
        assert!(arguments.len() == 1);
        let (_, arg) = arguments.pop().unwrap();
        Ok(Data::Json(
            serde_json::to_vec(&serde_json::json!([{"args": arg}])).unwrap().into(),
        ))
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

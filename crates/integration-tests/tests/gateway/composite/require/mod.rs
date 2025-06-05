mod batch;
mod single;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::ExtensionDirective;
use extension_catalog::Id;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::gateway::{AnyExtension, ResolverTestExtension, TestManifest};
use runtime::extension::{ArgumentsId, Data, Response};
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
            r#type: extension_catalog::Type::Resolver(Default::default()),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        });
        state.test.resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for Resolve {
    async fn resolve(
        &self,
        _directive: ExtensionDirective<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        mut arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        assert!(arguments.len() == 1);
        let (_, arg) = arguments.pop().unwrap();
        Response::from((self.resolve)(arg).map(|value| Data::Json(serde_json::to_vec(&value).unwrap().into())))
    }
}

fn gql_product() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        type Query {
            products: [Product]!
        }

        type Product @key(fields: "id") {
            id: ID!
            details: ProductDetails
            categories: [String!]!
        }

        type ProductDetails {
            code: String!
        }
        "#,
    )
    .with_resolver(
        "Query",
        "products",
        json!([
            {"id": "1", "details": {"code": "I1"}, "categories": ["C1", "C11"]},
            {"id": "2", "details": {"code": "I2"}, "categories": ["C2", "C22"]}
        ]),
    )
    .into_subgraph("gql")
}

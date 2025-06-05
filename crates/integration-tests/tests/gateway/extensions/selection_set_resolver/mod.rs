mod alias;
mod config;
mod errors;
mod lookup;
mod selection_set;
mod subgraph_schema;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::Subgraph;
use extension_catalog::{ExtensionId, Id};
use integration_tests::{
    gateway::{AnyExtension, Gateway, ResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data};

#[derive(Clone)]
pub struct StaticSelectionSetResolverExt {
    result: Result<Data, GraphqlError>,
}

impl StaticSelectionSetResolverExt {
    pub fn json(value: impl serde::Serialize) -> Self {
        Self {
            result: Ok(Data::Json(serde_json::to_vec(&value).unwrap().into())),
        }
    }

    pub fn json_bytes(bytes: &[u8]) -> Self {
        Self {
            result: Ok(Data::Json(bytes.to_vec().into())),
        }
    }

    pub fn error(error: GraphqlError) -> Self {
        Self { result: Err(error) }
    }
}

impl AnyExtension for StaticSelectionSetResolverExt {
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
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for StaticSelectionSetResolverExt {
    async fn resolve(
        &self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError> {
        self.result.clone()
    }
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @init

                scalar JSON

                type Query {
                    test: JSON
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(1))
            .build()
            .await;

        let response = engine.post("query { test }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": 1
          }
        }"#);
    })
}

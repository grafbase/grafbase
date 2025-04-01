use std::sync::Arc;

use engine::{Engine, GraphqlError};
use engine_schema::Subgraph;
use extension_catalog::{ExtensionId, Id};
use integration_tests::{
    federation::{AnyExtension, EngineExt, SubQueryResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data};

#[derive(Clone)]
pub struct StaticSubQueryResolverExt {
    data: Result<Data, GraphqlError>,
}

impl StaticSubQueryResolverExt {
    pub fn json(value: impl serde::Serialize) -> Self {
        Self {
            data: Ok(Data::JsonBytes(serde_json::to_vec(&value).unwrap())),
        }
    }
}

impl AnyExtension for StaticSubQueryResolverExt {
    fn register(self, state: &mut integration_tests::federation::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "static".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::SubQueryResolver(Default::default()),
            sdl: Some(r#"directive @init on SCHEMA"#),
        });
        state.test.subquery_resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn SubQueryResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl SubQueryResolverTestExtension for StaticSubQueryResolverExt {
    async fn resolve_field(
        &self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError> {
        self.data.clone()
    }
}

#[test]
fn basic() {
    runtime().block_on(async {
        let engine = Engine::builder()
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
            .with_extension(StaticSubQueryResolverExt::json(1))
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

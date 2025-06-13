mod nested;
mod subscription;

use std::sync::Arc;

use engine_schema::ExtensionDirective;
use extension_catalog::Id;
use integration_tests::{
    gateway::{AnyExtension, Gateway, ResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data, Response};

#[derive(Clone)]
pub enum ResolverExt {
    Response(Response),
    EchoData,
}

impl ResolverExt {
    pub fn json(value: impl serde::Serialize) -> Self {
        Self::Response(Response::data(Data::Json(serde_json::to_vec(&value).unwrap().into())))
    }

    pub fn echo_data() -> Self {
        Self::EchoData
    }
}

impl AnyExtension for ResolverExt {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "resolver".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                directives: Some(vec!["resolve".into()]),
            }),
            sdl: Some(
                r#"
                scalar JSON
                directive @resolve(data: JSON) on FIELD_DEFINITION
                "#,
            ),
        });
        state.test.resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for ResolverExt {
    async fn resolve(
        &self,
        _directive: ExtensionDirective<'_>,
        prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        match self {
            Self::Response(response) => response.clone(),
            Self::EchoData => {
                let value: serde_json::Value = serde_json::from_slice(prepared_data).unwrap();
                Response::data(Data::Json(serde_json::to_vec(&value["data"]).unwrap().into()))
            }
        }
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
                    @link(url: "resolver-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test(input: String): JSON @resolve
                }
                "#,
            )
            .with_extension(ResolverExt::json("hi!"))
            .build()
            .await;

        let response = engine.post("query { test(input: \"hi!\") }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": "hi!"
          }
        }
        "#);
    })
}

#[test]
fn wasm_basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-17-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test(input: String): JSON @resolve(data: {value: 1})
                }
                "#,
            )
            .with_extension("resolver-17")
            .build()
            .await;

        let response = engine.post("query { test(input: \"hi!\") }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "args": {
                "input": "hi!"
              },
              "config": {
                "key": null
              },
              "directive": {
                "data": {
                  "value": 1
                }
              }
            }
          }
        }
        "#);
    })
}

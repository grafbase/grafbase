mod nested;
mod sdk17;
mod sdk21;
mod subscription;

use std::sync::Arc;

use engine::{ErrorCode, GraphqlError};
use engine_schema::ExtensionDirective;
use extension_catalog::Id;
use integration_tests::{
    gateway::{AnyExtension, Gateway, ResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data, DynField, Response};

#[derive(Clone)]
pub enum ResolverExt {
    Response(Response),
    Callback(Arc<dyn Fn(String, serde_json::Value) -> serde_json::Value + Send + Sync + 'static>),
    EchoData,
    EchoHeader(String),
}

impl ResolverExt {
    pub fn json(value: impl serde::Serialize) -> Self {
        Self::Response(Response::data(Data::Json(serde_json::to_vec(&value).unwrap().into())))
    }

    pub fn echo_data() -> Self {
        Self::EchoData
    }
    pub fn callback<F>(callback: F) -> Self
    where
        F: Fn(String, serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        Self::Callback(Arc::new(callback))
    }

    pub fn echo_header(name: impl Into<String>) -> Self {
        Self::EchoHeader(name.into())
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
    async fn prepare<'ctx>(
        &self,
        _directive: ExtensionDirective<'ctx>,
        directive_arguments: serde_json::Value,
        field: Box<dyn DynField<'ctx>>,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self {
            Self::Response(_) | Self::EchoData | Self::EchoHeader(_) => serde_json::to_vec(&directive_arguments),
            Self::Callback(_) => serde_json::to_vec(&serde_json::json!({
                "name": format!("{}.{}", field.definition().parent_entity().name(), field.definition().name()),
                "arguments": directive_arguments,
            })),
        }
        .map_err(|e| {
            GraphqlError::new(
                format!("Failed to serialize directive arguments: {e}"),
                ErrorCode::ExtensionError,
            )
        })
    }

    async fn resolve(
        &self,
        _directive: ExtensionDirective<'_>,
        prepared_data: &[u8],
        headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        match self {
            Self::Response(response) => response.clone(),
            Self::EchoData => {
                let value: serde_json::Value = serde_json::from_slice(prepared_data).unwrap();
                Response::data(Data::Json(serde_json::to_vec(&value["data"]).unwrap().into()))
            }
            Self::Callback(callback) => {
                let value: serde_json::Value = serde_json::from_slice(prepared_data).unwrap();
                let result = callback(value["name"].as_str().unwrap().to_string(), value["arguments"].clone());
                Response::data(Data::Json(serde_json::to_vec(&result).unwrap().into()))
            }
            Self::EchoHeader(name) => {
                let value = headers.get(name).map(|value| value.to_str().unwrap_or_default());
                Response::data(Data::Json(serde_json::to_vec(&value).unwrap().into()))
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

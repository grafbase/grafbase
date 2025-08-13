use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::ExtensionDirective;
use extension_catalog::Id;
use futures::{StreamExt as _, stream::BoxStream};
use integration_tests::{
    gateway::{AnyExtension, Gateway, ResolverTestExtension, TestManifest},
    runtime,
};
use runtime::extension::{ArgumentsId, Data, Response};

#[derive(Clone, Default)]
pub struct SubscriptionExt {
    items: Vec<Response>,
}

impl SubscriptionExt {
    pub fn event(mut self, value: impl serde::Serialize) -> Self {
        self.items
            .push(Response::data(Data::Json(serde_json::to_vec(&value).unwrap().into())));
        self
    }
}

impl AnyExtension for SubscriptionExt {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "sub".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                directives: Some(vec!["resolve".into()]),
            }),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        });
        state.test.resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn ResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl ResolverTestExtension for SubscriptionExt {
    async fn resolve(
        &self,
        _directive: ExtensionDirective<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        Response::error(GraphqlError::internal_extension_error())
    }

    async fn resolve_subscription<'ctx>(
        &self,
        _directive: ExtensionDirective<'ctx>,
        _prepared_data: &'ctx [u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> BoxStream<'ctx, Response> {
        futures::stream::iter(self.items.clone()).boxed()
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
                    @link(url: "sub-1.0.0", import: ["@resolve"])

                scalar JSON

                type Subscription {
                    test: JSON @resolve
                }
                "#,
            )
            .with_extension(SubscriptionExt::default().event(1).event(2))
            .build()
            .await;

        let response = engine
            .post("subscription { test }")
            .into_sse_stream()
            .await
            .collect()
            .await;
        insta::assert_json_snapshot!(response.messages, @r#"
        [
          {
            "data": {
              "test": 1
            }
          },
          {
            "data": {
              "test": 2
            }
          }
        ]
        "#);
    })
}

use engine::Engine;
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestFieldResolvereExtension},
    runtime,
};
use runtime::{
    error::PartialGraphqlError,
    extension::ExtensionDirective,
    hooks::{DynHookContext, EdgeDefinition},
};

#[test]
fn simple_extension() {
    let tmpdir = tempfile::tempdir().unwrap();

    struct Ext;

    #[async_trait::async_trait]
    impl TestFieldResolvereExtension for Ext {
        async fn resolve<'a>(
            &self,
            _context: &DynHookContext,
            _field: EdgeDefinition<'a>,
            _directive: ExtensionDirective<'a, serde_json::Value>,
            inputs: Vec<serde_json::Value>,
        ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
            Ok(vec![
                Ok(serde_json::json!({
                    "greeting": "Hi"
                }));
                inputs.len()
            ])
        }
    }

    let response = runtime().block_on(async move {
        let origin = url::Url::from_file_path(tmpdir.path()).unwrap();
        let engine = Engine::builder()
            .with_federated_sdl(&format!(
                r#"
                    enum extension__Link {{
                        REST @extension__link(url: "{}")
                    }}

                    enum join__Graph {{
                        A @join__graph(name: "a")
                    }}

                    extend type Query {{
                        greeting(name: String): String @extension__directive(graph: A, extension: REST, name: "rest")
                    }}
                    "#,
                origin
            ))
            .with_extensions(|ext| {
                ext.with_field_resolver(
                    tmpdir.path(),
                    Id {
                        name: "test".to_string(),
                        version: "1.0.0".parse().unwrap(),
                    },
                    &["rest"],
                    Ext,
                )
            })
            .build()
            .await;

        engine.post("query { greeting }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": "Hi"
      }
    }
    "#);
}

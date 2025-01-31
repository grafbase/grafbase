use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    tests::create_log_channel, ComponentLoader, Directive, ExtensionType, ExtensionsComponentInstance, FieldDefinition,
    SharedContext,
};
use gateway_config::WasiExtensionsConfig;
use grafbase_telemetry::otel::opentelemetry::trace::TraceId;

#[tokio::test]
async fn simple_resolver() {
    #[derive(serde::Serialize)]
    struct SchemaArgs {
        id: usize,
    }

    #[derive(serde::Serialize)]
    struct FieldArgs<'a> {
        name: &'a str,
    }

    let config = WasiExtensionsConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/simple_resolver.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let (access_log, _) = create_log_channel();
    let loader = ComponentLoader::extensions(String::new(), config).unwrap().unwrap();
    let schema_directive = Directive::new("schemaArgs".into(), "mySubgraph".into(), &SchemaArgs { id: 10 });

    let mut extension =
        ExtensionsComponentInstance::new(&loader, ExtensionType::Resolver, vec![schema_directive], access_log)
            .await
            .unwrap();

    let context = SharedContext::new(Arc::new(HashMap::new()), TraceId::INVALID);

    let field_directive = Directive::new("myDirective".into(), "mySubgraph".into(), &FieldArgs { name: "cat" });

    let definition = FieldDefinition {
        type_name: "Query".into(),
        name: "cats".into(),
    };

    let output = extension
        .resolve_field(context, field_directive, definition, Vec::<serde_json::Value>::new())
        .await
        .unwrap();

    let result: serde_json::Value = output.serialize_outputs().pop().unwrap().unwrap();

    insta::assert_json_snapshot!(&result, @r#"
    {
      "id": 10,
      "name": "cat"
    }
    "#);
}

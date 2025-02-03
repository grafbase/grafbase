use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    tests::create_log_channel, ComponentLoader, Directive, ExtensionType, ExtensionsComponentInstance, FieldDefinition,
    SharedContext,
};
use gateway_config::WasiExtensionsConfig;
use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use http::{HeaderMap, HeaderValue};
use serde_json::json;

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

    let mut extension = ExtensionsComponentInstance::new(
        &loader,
        ExtensionType::Resolver,
        vec![schema_directive],
        &"",
        access_log,
    )
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

#[tokio::test]
async fn single_call_caching_auth() {
    let config = WasiExtensionsConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let (access_log, _) = create_log_channel();
    let loader = ComponentLoader::extensions(String::new(), config).unwrap().unwrap();

    let config = json!({
        "cache_config": "test"
    });

    let mut extension =
        ExtensionsComponentInstance::new(&loader, ExtensionType::Authentication, Vec::new(), &config, access_log)
            .await
            .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("valid"));

    let (headers, output): (_, serde_json::Value) = extension.authenticate(headers).await.unwrap();

    assert!(headers.len() == 1);
    assert_eq!(Some(&HeaderValue::from_static("valid")), headers.get("Authorization"));

    insta::assert_json_snapshot!(output, @r#"
    {
      "key": "valid"
    }
    "#);
}

#[tokio::test]
async fn single_call_caching_auth_invalid() {
    let config = WasiExtensionsConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let (access_log, _) = create_log_channel();
    let loader = ComponentLoader::extensions(String::new(), config).unwrap().unwrap();

    let config = json!({
        "cache_config": "test"
    });

    let mut extension =
        ExtensionsComponentInstance::new(&loader, ExtensionType::Authentication, Vec::new(), &config, access_log)
            .await
            .unwrap();

    let result = extension
        .authenticate::<serde_json::Value>(HeaderMap::new())
        .await
        .unwrap_err();

    insta::assert_debug_snapshot!(result, @r#"
    Guest(
        ErrorResponse {
            status_code: 401,
            errors: [],
        },
    )
    "#);
}

#[tokio::test]
async fn multiple_cache_calls() {
    let config = WasiExtensionsConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let (access_log, _) = create_log_channel();
    let loader = ComponentLoader::extensions(String::new(), config).unwrap().unwrap();

    let config = json!({
        "cache_config": "test"
    });

    let mut extension =
        ExtensionsComponentInstance::new(&loader, ExtensionType::Authentication, Vec::new(), &config, access_log)
            .await
            .unwrap();

    for _ in 0..20 {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("valid"));

        let (_, output): (_, serde_json::Value) = extension.authenticate(headers).await.unwrap();

        insta::allow_duplicates! {
            insta::assert_json_snapshot!(output, @r#"
            {
              "key": "valid"
            }
            "#);
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("nonvalid"));

    let (_, output): (_, serde_json::Value) = extension.authenticate(headers).await.unwrap();

    insta::allow_duplicates! {
        insta::assert_json_snapshot!(output, @r#"
        {
          "key": "nonvalid"
        }
        "#);
    }
}

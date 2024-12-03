use std::{collections::HashMap, sync::Arc};

use crate::{
    error::guest::ErrorResponse, hooks::ComponentInstance, CacheStatus, ChannelLogReceiver, ChannelLogSender,
    ComponentLoader, Config, EdgeDefinition, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest,
    GuestError, NodeDefinition, SharedContext, SubgraphResponse,
};
use expect_test::expect;
use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use http::{HeaderMap, HeaderValue};
use indoc::{formatdoc, indoc};
use serde_json::json;
use tempfile::TempDir;
use wiremock::{matchers::method, ResponseTemplate};

fn create_log_channel() -> (ChannelLogSender, ChannelLogReceiver) {
    crate::create_log_channel(
        false,
        grafbase_telemetry::metrics::meter_from_global_provider()
            .i64_up_down_counter("grafbase.gateway.access_log.pending")
            .build(),
    )
}

#[tokio::test]
async fn missing_wasm() {
    let config = Config::default();
    assert!(!config.location.exists());

    let loader = ComponentLoader::new(config).unwrap();
    assert!(loader.is_none());
}

#[tokio::test]
async fn missing_hook() {
    // the guest code in examples/missing_hook/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/missing_hook.wasm"
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let (access_log, _) = create_log_channel();
    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();

    let (context, headers) = hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    assert_eq!(HeaderMap::new(), headers);
    assert_eq!(HashMap::new(), context);
}

#[tokio::test]
async fn simple_no_io() {
    // the guest code in examples/simple/src/lib.rs

    std::env::set_var("GRAFBASE_WASI_TEST", "meow");

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/simple.wasm"
        environment_variables = true
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut context = HashMap::new();
    context.insert("kekw".to_string(), "lol".to_string());

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, headers) = hook.on_gateway_request(context, HeaderMap::new()).await.unwrap();

    assert_eq!(Some(&HeaderValue::from_static("call")), headers.get("direct"));
    assert_eq!(Some(&HeaderValue::from_static("meow")), headers.get("fromEnv"));
    assert_eq!(Some("direct"), context.get("call").map(|v| v.as_str()));
}

#[tokio::test]
async fn dir_access_read_only() {
    // the guest code in examples/dir_access/src/lib.rs

    let dir = TempDir::new().unwrap();
    let path = dir.path();
    let path_str = path.to_str().unwrap().escape_default();

    let config = formatdoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/dir_access.wasm"
        stdout = true
        stderr = true

        [[preopened_directories]]
        host_path = "{path_str}"
        guest_path = "."
        read_permission = true
        write_permission = false
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location.exists());

    std::fs::write(path.join("contents.txt"), "test string").unwrap();

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (_, headers) = hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    assert_eq!(
        Some(&HeaderValue::from_static("test string")),
        headers.get("READ_CONTENTS")
    );

    assert!(!path.join("guest_write.txt").exists());
}

#[tokio::test]
async fn dir_access_write() {
    // the guest code in examples/dir_access/src/lib.rs

    let dir = TempDir::new().unwrap();
    let path = dir.path();
    let path_str = path.to_str().unwrap().escape_default();

    let config = formatdoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/dir_access.wasm"
        stdout = true
        stderr = true

        [[preopened_directories]]
        host_path = "{path_str}"
        guest_path = "."
        read_permission = true
        write_permission = true
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location.exists());

    std::fs::write(path.join("contents.txt"), "test string").unwrap();

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    let path = path.join("guest_write.txt");

    assert!(path.exists());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert_eq!("answer", &contents);
}

#[tokio::test]
async fn http_client() {
    // the guest code in examples/http_client/src/lib.rs

    let response = ResponseTemplate::new(200).set_body_string("kekw");
    let server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .respond_with(response)
        .mount(&server)
        .await;

    std::env::set_var("MOCK_SERVER_ADDRESS", format!("http://{}", server.address()));

    let config = formatdoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/http_client.wasm"
        environment_variables = true
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    assert_eq!(Some("kekw"), context.get("HTTP_RESPONSE").map(|s| s.as_str()));
}

#[tokio::test]
async fn guest_error() {
    // the guest code in examples/error/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/error.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();

    let error = hook
        .on_gateway_request(HashMap::new(), HeaderMap::new())
        .await
        .unwrap_err();

    let expected = ErrorResponse {
        status_code: 403,
        errors: vec![GuestError {
            message: String::from("not found"),
            extensions: vec![(String::from("my"), String::from("extension"))],
        }],
    };

    assert_eq!(Some(expected), error.into_guest_error());
}

#[tokio::test]
async fn authorize_edge_pre_execution_error() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (kv, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let context = SharedContext::new(Arc::new(kv), TraceId::INVALID);

    let value = json!({
        "value": "lol"
    });

    let error = hook
        .authorize_edge_pre_execution(
            context,
            definition,
            serde_json::to_string(&value).unwrap(),
            String::new(),
        )
        .await
        .unwrap_err();

    let expected = GuestError {
        message: String::from("not authorized"),
        extensions: vec![],
    };

    assert_eq!(Some(expected), error.into_guest_error());
}

#[tokio::test]
async fn authorize_edge_pre_execution_success() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let value = json!({
        "value": "kekw"
    });

    hook.authorize_edge_pre_execution(
        context,
        definition,
        serde_json::to_string(&value).unwrap(),
        String::new(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn authorize_node_pre_execution_error() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let metadata = json!({
        "role": "lol"
    });

    let error = hook
        .authorize_node_pre_execution(context, definition, serde_json::to_string(&metadata).unwrap())
        .await
        .unwrap_err();

    let expected = GuestError {
        message: String::from("not authorized"),
        extensions: vec![],
    };

    assert_eq!(Some(expected), error.into_guest_error());
}

#[tokio::test]
async fn authorize_node_pre_execution_success() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let metadata = json!({
        "role": "kekw"
    });

    hook.authorize_node_pre_execution(context, definition, serde_json::to_string(&metadata).unwrap())
        .await
        .unwrap();
}

#[tokio::test]
async fn authorize_parent_edge_post_execution() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let parents = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let metadata = json!({
        "role": "kekw"
    });

    let result = hook
        .authorize_parent_edge_post_execution(context, definition, parents, serde_json::to_string(&metadata).unwrap())
        .await
        .unwrap();

    let expected = expect![[r#"
        [
            Ok(
                (),
            ),
            Err(
                GuestError {
                    extensions: [],
                    message: "not authorized",
                },
            ),
        ]
    "#]];

    expected.assert_debug_eq(&result);
}

#[tokio::test]
async fn authorize_edge_node_post_execution() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let nodes = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let result = hook
        .authorize_edge_node_post_execution(context, definition, nodes, String::new())
        .await
        .unwrap();

    let expected = expect![[r#"
        [
            Ok(
                (),
            ),
            Err(
                GuestError {
                    extensions: [],
                    message: "not authorized",
                },
            ),
        ]
    "#]];

    expected.assert_debug_eq(&result);
}

#[tokio::test]
async fn authorize_edge_post_execution() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/authorization.wasm"
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let parent1 = serde_json::to_string(&json!({ "id": 1 })).unwrap();

    let nodes1 = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let parent2 = serde_json::to_string(&json!({ "id": 2 })).unwrap();

    let nodes2 = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let result = hook
        .authorize_edge_post_execution(
            context,
            definition,
            vec![(parent1, nodes1), (parent2, nodes2)],
            String::new(),
        )
        .await
        .unwrap();

    let expected = expect![[r#"
        [
            Ok(
                (),
            ),
            Err(
                GuestError {
                    extensions: [],
                    message: "not authorized",
                },
            ),
            Ok(
                (),
            ),
            Err(
                GuestError {
                    extensions: [],
                    message: "not authorized",
                },
            ),
        ]
    "#]];

    expected.assert_debug_eq(&result);
}

#[tokio::test]
async fn on_subgraph_request() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/subgraph_request.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Hi", HeaderValue::from_static("Rusty"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, _) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();
    let (context, headers) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let headers = hook
        .on_subgraph_request(
            context,
            "dummy",
            http::Method::POST,
            &"http://example.com".parse().unwrap(),
            headers,
        )
        .await
        .unwrap();

    let everything = headers
        .get("everything")
        .map(|value| URL_SAFE_NO_PAD.decode(value.to_str().unwrap()).unwrap())
        .unwrap_or_default();
    let value = serde_json::from_slice::<serde_json::Value>(&everything).unwrap();
    insta::assert_json_snapshot!(value, @r#"
    {
      "subgraph_name": "dummy",
      "method": "POST",
      "url": "http://example.com/",
      "headers": [
        [
          "hi",
          "Rusty"
        ]
      ]
    }
    "#);

    let context = HashMap::from_iter([("should-fail".into(), "yes".into())]);

    let context = SharedContext::new(Arc::new(context), TraceId::INVALID);

    let error = hook
        .on_subgraph_request(
            context,
            "dummy",
            http::Method::POST,
            &"http://example.com".parse().unwrap(),
            headers,
        )
        .await
        .unwrap_err();

    insta::assert_debug_snapshot!(error, @r#"
    Guest(
        GuestError {
            extensions: [],
            message: "failure",
        },
    )
    "#);
}

#[tokio::test]
async fn response_hooks() {
    let config = indoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/response_hooks.wasm"
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let (access_log, receiver) = create_log_channel();
    let mut hook = ComponentInstance::new(&loader, access_log).await.unwrap();

    let context = SharedContext::new(Arc::new(HashMap::new()), TraceId::INVALID);

    let request = ExecutedSubgraphRequest {
        subgraph_name: String::from("kekw"),
        method: String::from("POST"),
        url: String::from("https://example.com"),
        total_duration_ms: 10,
        has_errors: false,
        executions: vec![crate::SubgraphRequestExecutionKind::Response(SubgraphResponse {
            connection_time_ms: 10,
            response_time_ms: 4,
            status_code: 200,
        })],
        cache_status: CacheStatus::Miss,
    };

    let subgraph_info = hook.on_subgraph_response(context.clone(), request).await.unwrap();

    let operation = ExecutedOperation {
        duration_ms: 5,
        status: crate::GraphqlResponseStatus::Success,
        on_subgraph_response_outputs: vec![subgraph_info],
        name: Some(String::from("kekw")),
        document: String::from("query { me { 1 } }"),
        prepare_duration_ms: 3,
        cached_plan: false,
    };

    let op_info = hook.on_operation_response(context.clone(), operation).await.unwrap();

    let request = ExecutedHttpRequest {
        method: String::from("POST"),
        url: String::from("https://example.com/graphql"),
        status_code: 200,
        on_operation_response_outputs: vec![op_info],
    };

    hook.on_http_response(context.clone(), request).await.unwrap();

    let data = receiver.recv().unwrap().into_data().unwrap();
    let data: serde_json::Value = serde_json::from_slice(&data).unwrap();

    insta::assert_json_snapshot!(&data, @r###"
    {
      "method": "POST",
      "url": "https://example.com/graphql",
      "status_code": 200,
      "trace_id": "00000000000000000000000000000000",
      "operations": [
        {
          "name": "kekw",
          "document": "query { me { 1 } }",
          "prepare_duration": 3,
          "cached": false,
          "duration": 5,
          "status": "Success",
          "subgraphs": [
            {
              "subgraph_name": "kekw",
              "method": "POST",
              "url": "https://example.com",
              "responses": [
                {
                  "Responsed": {
                    "connection_time": 10,
                    "response_time": 4,
                    "status_code": 200
                  }
                }
              ],
              "total_duration": 10,
              "has_errors": false,
              "cached": false
            }
          ]
        }
      ]
    }
    "###);
}

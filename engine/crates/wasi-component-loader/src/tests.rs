use std::{collections::HashMap, sync::Arc};

use crate::{
    create_log_channel,
    hooks::{
        response::{CacheStatus, SubgraphResponseInfo},
        subgraph::SubgraphComponentInstance,
    },
    AuthorizationComponentInstance, ComponentLoader, Config, EdgeDefinition, ExecutedGatewayRequest,
    ExecutedHttpRequest, ExecutedSubgraphRequest, GatewayComponentInstance, GuestError, NodeDefinition, Operation,
    RecycleableComponentInstance, ResponsesComponentInstance, SharedContext,
};
use expect_test::expect;
use http::{HeaderMap, HeaderValue};
use indoc::{formatdoc, indoc};
use serde_json::json;
use tempdir::TempDir;
use wiremock::{matchers::method, ResponseTemplate};

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

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();

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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, headers) = hook.on_gateway_request(context, HeaderMap::new()).await.unwrap();

    assert_eq!(Some(&HeaderValue::from_static("call")), headers.get("direct"));
    assert_eq!(Some(&HeaderValue::from_static("meow")), headers.get("fromEnv"));
    assert_eq!(Some("direct"), context.get("call").map(|v| v.as_str()));
}

#[tokio::test]
async fn dir_access_read_only() {
    // the guest code in examples/dir_access/src/lib.rs

    let dir = TempDir::new("test").unwrap();
    let path = dir.path();
    let path_str = path.to_str().unwrap();

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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
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

    let dir = TempDir::new("test").unwrap();
    let path = dir.path();
    let path_str = path.to_str().unwrap();

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
    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    let path = path.join("guest_write.txt");

    assert!(path.exists());

    let contents = std::fs::read_to_string(&path).unwrap();
    assert_eq!("answer", &contents);
}

#[tokio::test]
async fn networking() {
    // the guest code in examples/networking/src/lib.rs

    let response = ResponseTemplate::new(200).set_body_string("kekw");
    let server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .respond_with(response)
        .mount(&server)
        .await;

    std::env::set_var("MOCK_SERVER_ADDRESS", format!("http://{}", server.address()));

    let config = formatdoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/networking.wasm"
        environment_variables = true
        stdout = true
        stderr = true
        networking = true
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), HeaderMap::new()).await.unwrap();

    assert_eq!(Some("kekw"), context.get("HTTP_RESPONSE").map(|s| s.as_str()));
}

#[tokio::test]
async fn networking_no_network() {
    // the guest code in examples/networking/src/lib.rs

    let response = ResponseTemplate::new(200).set_body_string("kekw");
    let server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .respond_with(response)
        .mount(&server)
        .await;

    std::env::set_var("MOCK_SERVER_ADDRESS", format!("http://{}", server.address()));

    let config = formatdoc! {r#"
        location = "examples/target/wasm32-wasip1/debug/networking.wasm"
        environment_variables = true
        stdout = true
        stderr = true
        networking = false
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location.exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let error = GatewayComponentInstance::new(&loader).await.unwrap_err();

    let expected = expect![
        "component imports instance `wasi:http/types@0.2.0`, but a matching implementation was not found in the linker"
    ];

    expected.assert_eq(&error.to_string());
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
    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let error = hook
        .on_gateway_request(HashMap::new(), HeaderMap::new())
        .await
        .unwrap_err();

    let expected = GuestError {
        message: String::from("not found"),
        extensions: vec![(String::from("my"), String::from("extension"))],
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (kv, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(kv), access_log);

    let error = hook
        .authorize_edge_pre_execution(context, definition, String::new(), String::new())
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

    hook.authorize_edge_pre_execution(context, definition, String::from("kekw"), String::new())
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

    let error = hook
        .authorize_node_pre_execution(context, definition, String::new())
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

    hook.authorize_node_pre_execution(context, definition, String::from("kekw"))
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let parents = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

    let result = hook
        .authorize_parent_edge_post_execution(context, definition, parents, String::new())
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let nodes = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

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
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location.exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, _) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationComponentInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let nodes1 = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let nodes2 = vec![
        serde_json::to_string(&json!({ "value": "kekw" })).unwrap(),
        serde_json::to_string(&json!({ "value": "lol" })).unwrap(),
    ];

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

    let result = hook
        .authorize_edge_post_execution(
            context,
            definition,
            vec![(String::new(), nodes1), (String::new(), nodes2)],
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

    let mut hook = GatewayComponentInstance::new(&loader).await.unwrap();
    let (context, headers) = hook.on_gateway_request(HashMap::new(), headers).await.unwrap();

    let mut hook = SubgraphComponentInstance::new(&loader).await.unwrap();

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

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
    insta::assert_json_snapshot!(value, @r###"
    {
      "headers": [
        [
          "hi",
          "Rusty"
        ]
      ],
      "method": "POST",
      "subgraph_name": "dummy",
      "url": "http://example.com/"
    }
    "###);

    let context = HashMap::from_iter([("should-fail".into(), "yes".into())]);

    let (access_log, _) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(context), access_log);

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

    insta::assert_debug_snapshot!(error, @r###"
    Guest(
        GuestError {
            extensions: [],
            message: "failure",
        },
    )
    "###);
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

    let mut hook = ResponsesComponentInstance::new(&loader).await.unwrap();

    let (access_log, receiver) = create_log_channel(false);
    let context = SharedContext::new(Arc::new(HashMap::new()), access_log);

    let request = ExecutedSubgraphRequest {
        subgraph_name: String::from("kekw"),
        method: String::from("POST"),
        url: String::from("https://example.com"),
        total_duration: 10,
        has_errors: false,
        response_infos: vec![SubgraphResponseInfo {
            connection_time: 10,
            response_time: 4,
            status_code: 200,
        }],
        cache_status: CacheStatus::Miss,
    };

    let subgraph_info = hook.on_subgraph_response(context.clone(), request).await.unwrap();

    let request = ExecutedGatewayRequest {
        duration: 5,
        status: crate::GraphqlResponseStatus::Success,
        on_subgraph_request_outputs: vec![subgraph_info],
    };

    let operation = Operation {
        name: Some(String::from("kekw")),
        document: String::from("query { me { 1 } }"),
        prepare_duration: 3,
        cached: false,
    };

    let op_info = hook
        .on_gateway_response(context.clone(), operation, request)
        .await
        .unwrap();

    let request = ExecutedHttpRequest {
        method: String::from("POST"),
        url: String::from("https://example.com/graphql"),
        status_code: 200,
        on_gateway_response_outputs: vec![op_info],
    };

    hook.on_http_response(context.clone(), request).await.unwrap();

    let data = receiver.recv().unwrap().into_data().unwrap();
    let data: serde_json::Value = serde_json::from_slice(&data).unwrap();

    insta::assert_json_snapshot!(&data, @r###"
    {
      "method": "POST",
      "url": "https://example.com/graphql",
      "status_code": 200,
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
              "connection_times": [
                10
              ],
              "response_times": [
                4
              ],
              "status_codes": [
                200
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

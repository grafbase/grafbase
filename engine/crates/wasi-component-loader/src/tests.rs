use std::{collections::HashMap, sync::Arc};

use crate::{
    AuthorizationHookInstance, ComponentLoader, Config, EdgeDefinition, ErrorResponse, GatewayHookInstance,
    NodeDefinition,
};
use expect_test::expect;
use http::{HeaderMap, HeaderValue};
use indoc::{formatdoc, indoc};
use tempdir::TempDir;
use wiremock::{matchers::method, ResponseTemplate};

#[tokio::test]
async fn missing_wasm() {
    let config = Config::default();
    assert!(!config.location().exists());

    let loader = ComponentLoader::new(config).unwrap();
    assert!(loader.is_none());
}

#[tokio::test]
async fn missing_hook() {
    // the guest code in examples/missing_hook/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/missing_hook.wasm"
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();

    let (context, headers) = hook.call(HashMap::new(), HeaderMap::new()).await.unwrap();

    assert_eq!(HeaderMap::new(), headers);
    assert_eq!(HashMap::new(), context);
}

#[tokio::test]
async fn simple_no_io() {
    // the guest code in examples/simple/src/lib.rs

    std::env::set_var("GRAFBASE_WASI_TEST", "meow");

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/simple.wasm"
        environment_variables = true
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut context = HashMap::new();
    context.insert("kekw".to_string(), "lol".to_string());

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, headers) = hook.call(context, HeaderMap::new()).await.unwrap();

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
        location = "examples/target/wasm32-wasi/debug/dir_access.wasm"
        stdout = true
        stderr = true

        [[preopened_directories]]
        host_path = "{path_str}"
        guest_path = "."
        read_permission = true
        write_permission = false
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location().exists());

    std::fs::write(path.join("contents.txt"), "test string").unwrap();

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (_, headers) = hook.call(HashMap::new(), HeaderMap::new()).await.unwrap();

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
        location = "examples/target/wasm32-wasi/debug/dir_access.wasm"
        stdout = true
        stderr = true

        [[preopened_directories]]
        host_path = "{path_str}"
        guest_path = "."
        read_permission = true
        write_permission = true
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location().exists());

    std::fs::write(path.join("contents.txt"), "test string").unwrap();

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    hook.call(HashMap::new(), HeaderMap::new()).await.unwrap();

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
        location = "examples/target/wasm32-wasi/debug/networking.wasm"
        environment_variables = true
        stdout = true
        stderr = true
        networking = true
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, _) = hook.call(HashMap::new(), HeaderMap::new()).await.unwrap();

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
        location = "examples/target/wasm32-wasi/debug/networking.wasm"
        environment_variables = true
        stdout = true
        stderr = true
        networking = false
    "#};

    let config: Config = toml::from_str(&config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let error = GatewayHookInstance::new(&loader).await.unwrap_err();

    let expected = expect![
        "component imports instance `wasi:http/types@0.2.0`, but a matching implementation was not found in the linker"
    ];

    expected.assert_eq(&error.to_string());
}

#[tokio::test]
async fn guest_error() {
    // the guest code in examples/error/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/error.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let error = hook.call(HashMap::new(), HeaderMap::new()).await.unwrap_err();

    let expected = ErrorResponse {
        message: String::from("not found"),
        extensions: vec![(String::from("my"), String::from("extension"))],
    };

    assert_eq!(Some(expected), error.into_user_error());
}

#[tokio::test]
async fn authorize_edge_pre_execution_error() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, _) = hook.call(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationHookInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    let error = hook
        .authorize_edge_pre_execution(Arc::new(context), definition, String::new(), String::new())
        .await
        .unwrap_err();

    let expected = expect![[r#"
        User(
            ErrorResponse {
                extensions: [],
                message: "not authorized",
            },
        )
    "#]];

    expected.assert_debug_eq(&error);
}

#[tokio::test]
async fn authorize_edge_pre_execution_success() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, _) = hook.call(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationHookInstance::new(&loader).await.unwrap();

    let definition = EdgeDefinition {
        parent_type_name: String::new(),
        field_name: String::new(),
    };

    hook.authorize_edge_pre_execution(Arc::new(context), definition, String::from("kekw"), String::new())
        .await
        .unwrap();
}

#[tokio::test]
async fn authorize_node_pre_execution_error() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, _) = hook.call(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationHookInstance::new(&loader).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    let error = hook
        .authorize_node_pre_execution(Arc::new(context), definition, String::new())
        .await
        .unwrap_err();

    let expected = expect![[r#"
        User(
            ErrorResponse {
                extensions: [],
                message: "not authorized",
            },
        )
    "#]];

    expected.assert_debug_eq(&error);
}

#[tokio::test]
async fn authorize_node_pre_execution_success() {
    // the guest code in examples/authorization/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/authorization.wasm"
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("kekw"));

    let loader = ComponentLoader::new(config).unwrap().unwrap();

    let mut hook = GatewayHookInstance::new(&loader).await.unwrap();
    let (context, _) = hook.call(HashMap::new(), headers).await.unwrap();

    let mut hook = AuthorizationHookInstance::new(&loader).await.unwrap();

    let definition = NodeDefinition {
        type_name: String::new(),
    };

    hook.authorize_node_pre_execution(Arc::new(context), definition, String::from("kekw"))
        .await
        .unwrap();
}

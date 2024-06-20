use crate::{ComponentLoader, Config, ErrorResponse};
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
async fn missing_callback() {
    // the guest code in examples/missing_callback/src/lib.rs

    let config = indoc! {r#"
        location = "examples/target/wasm32-wasi/debug/missing_callback.wasm"
        stdout = true
        stderr = true
    "#};

    let config: Config = toml::from_str(config).unwrap();
    assert!(config.location().exists());

    let loader = ComponentLoader::new(config).unwrap().unwrap();
    let headers = loader.on_gateway_request(HeaderMap::new()).await.unwrap();

    assert_eq!(HeaderMap::new(), headers);
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
    let headers = loader.on_gateway_request(HeaderMap::new()).await.unwrap();

    assert_eq!(Some(&HeaderValue::from_static("call")), headers.get("direct"));
    assert_eq!(Some(&HeaderValue::from_static("meow")), headers.get("fromEnv"));
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
    let headers = loader.on_gateway_request(HeaderMap::new()).await.unwrap();

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
    loader.on_gateway_request(HeaderMap::new()).await.unwrap();

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
    let headers = loader.on_gateway_request(HeaderMap::new()).await.unwrap();

    assert_eq!(Some(&HeaderValue::from_static("kekw")), headers.get("HTTP_RESPONSE"));
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
    let error = loader.on_gateway_request(HeaderMap::new()).await.unwrap_err();

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
    let error = loader.on_gateway_request(HeaderMap::new()).await.unwrap_err();

    let expected = ErrorResponse {
        message: String::from("not found"),
        extensions: vec![(String::from("my"), String::from("extension"))],
    };

    assert_eq!(Some(expected), error.into_user_error());
}

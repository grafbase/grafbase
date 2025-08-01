use std::{path::PathBuf, sync::Arc};

use crate::{
    ExtensionState, WasmContext,
    extension::{ExtensionConfig, ExtensionLoader, WasmConfig, build_engine},
};
use engine_schema::Schema;
use extension_catalog::{ExtensionId, TypeDiscriminants};
use futures::{
    StreamExt, TryStreamExt,
    stream::{FuturesOrdered, FuturesUnordered},
};
use http::{HeaderMap, HeaderValue, Request, Response};
use runtime::extension::Token;

const LATEST_SDK: semver::Version = semver::Version::new(0, 19, 0);

#[tokio::test]
async fn single_call_caching_auth() {
    let config = WasmConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let loader = load(ExtensionConfig {
        id: ExtensionId::from(0usize),
        r#type: TypeDiscriminants::Authentication,
        manifest_id: "caching_auth-1.0.0".parse().unwrap(),
        sdk_version: LATEST_SDK,
        pool: Default::default(),
        wasm: config,
        guest_config: toml::from_str(
            r#"
                cache_config = "test"
                "#,
        )
        .unwrap(),
        can_skip_sending_events: false,
        logging_filter: String::from("info"),
    })
    .await;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("valid"));

    let context = WasmContext::default();

    let (headers, token) = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(&context, headers.into())
        .await
        .unwrap()
        .unwrap();

    let headers = headers.into_inner().unwrap();

    assert!(headers.len() == 1);
    assert_eq!(Some(&HeaderValue::from_static("valid")), headers.get("Authorization"));

    let claims = match token {
        Token::Anonymous => serde_json::Value::Null,
        Token::Bytes(bytes) => serde_json::from_slice(&bytes).unwrap(),
    };

    insta::assert_json_snapshot!(claims, @r#"
    {
      "key": "default"
    }
    "#);
}

#[tokio::test]
async fn single_call_caching_auth_invalid() {
    let config = WasmConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());
    let loader = load(ExtensionConfig {
        id: ExtensionId::from(0usize),
        r#type: TypeDiscriminants::Authentication,
        manifest_id: "caching_auth-1.0.0".parse().unwrap(),
        sdk_version: LATEST_SDK,
        pool: Default::default(),
        wasm: config,
        guest_config: toml::from_str(
            r#"
                cache_config = "test"
                "#,
        )
        .unwrap(),
        can_skip_sending_events: false,
        logging_filter: String::from("info"),
    })
    .await;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("valid"));

    let context = WasmContext::default();

    let err = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(&context, http::HeaderMap::new().into())
        .await
        .unwrap()
        .err();

    insta::assert_debug_snapshot!(err, @r#"
    Some(
        ErrorResponse {
            status: 401,
            errors: [],
            headers: {},
        },
    )
    "#);
}

#[tokio::test]
async fn multiple_cache_calls() {
    let config = WasmConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/caching_auth.wasm"),
        networking: false,
        stdout: false,
        stderr: false,
        environment_variables: false,
    };

    assert!(config.location.exists());
    let loader = load(ExtensionConfig {
        id: ExtensionId::from(0usize),
        r#type: TypeDiscriminants::Authentication,
        manifest_id: "caching_auth-1.0.0".parse().unwrap(),
        sdk_version: LATEST_SDK,
        pool: Default::default(),
        wasm: config,
        guest_config: toml::from_str(
            r#"
                cache_config = "test"
                "#,
        )
        .unwrap(),
        can_skip_sending_events: false,
        logging_filter: String::from("info"),
    })
    .await;

    let mut tasks = FuturesOrdered::new();

    let extensions = (0..200)
        .map(|_| loader.instantiate())
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    for (i, mut extension) in extensions.into_iter().enumerate() {
        tasks.push_back(tokio::task::spawn(async move {
            let mut headers = HeaderMap::new();
            headers.insert("Authorization", HeaderValue::from_static("valid"));
            headers.insert("value", HeaderValue::from_str(&format!("value_{i}")).unwrap());

            let context = WasmContext::default();

            let (_, token) = extension.authenticate(&context, headers.into()).await.unwrap().unwrap();

            let claims = match token {
                Token::Anonymous => serde_json::Value::Null,
                Token::Bytes(bytes) => serde_json::from_slice(&bytes).unwrap(),
            };

            // only the first key comes from the cache.

            insta::allow_duplicates! {
                insta::assert_json_snapshot!(claims, @r#"
                {
                  "key": "value_0"
                }
                "#);
            }
        }))
    }

    while let Some(task) = tasks.next().await {
        task.unwrap();
    }

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("nonvalid"));

    let context = WasmContext::default();

    let (_, token) = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(&context, headers.into())
        .await
        .unwrap()
        .unwrap();

    let claims = match token {
        Token::Anonymous => serde_json::Value::Null,
        Token::Bytes(bytes) => serde_json::from_slice(&bytes).unwrap(),
    };

    insta::allow_duplicates! {
        insta::assert_json_snapshot!(claims, @r#"
        {
          "key": "default"
        }
        "#);
    }
}

#[tokio::test]
async fn on_request_hook() {
    let config = WasmConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/simple_hooks.wasm"),
        networking: false,
        stdout: true,
        stderr: true,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let loader = load(ExtensionConfig {
        id: ExtensionId::from(0usize),
        r#type: TypeDiscriminants::Hooks,
        manifest_id: "simple-hooks-1.0.0".parse().unwrap(),
        sdk_version: LATEST_SDK,
        pool: Default::default(),
        wasm: config,
        guest_config: toml::Value::Table(Default::default()),
        can_skip_sending_events: false,
        logging_filter: String::from("info"),
    })
    .await;

    let request = Request::builder().uri("https://example.com").body(()).unwrap();
    let (parts, _) = request.into_parts();

    loader
        .instantiate()
        .await
        .unwrap()
        .on_request(WasmContext::default(), parts)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn on_response_hook() {
    let config = WasmConfig {
        location: PathBuf::from("examples/target/wasm32-wasip2/debug/simple_hooks.wasm"),
        networking: false,
        stdout: true,
        stderr: true,
        environment_variables: false,
    };

    assert!(config.location.exists());

    let loader = load(ExtensionConfig {
        id: ExtensionId::from(0usize),
        r#type: TypeDiscriminants::Hooks,
        manifest_id: "simple-hooks-1.0.0".parse().unwrap(),
        sdk_version: LATEST_SDK,
        pool: Default::default(),
        wasm: config,
        guest_config: toml::Value::Table(Default::default()),
        can_skip_sending_events: false,
        logging_filter: String::from("info"),
    })
    .await;

    let response = Response::builder().status(200).body(()).unwrap();
    let (parts, _) = response.into_parts();

    loader
        .instantiate()
        .await
        .unwrap()
        .on_response(WasmContext::default(), parts)
        .await
        .unwrap()
        .unwrap();
}

async fn load(config: ExtensionConfig) -> ExtensionLoader {
    let engine = build_engine(Default::default()).unwrap();
    ExtensionLoader::new(
        &engine,
        Arc::new(Schema::from_sdl_or_panic("").await),
        Arc::new(ExtensionState::new(config)),
    )
    .unwrap()
}

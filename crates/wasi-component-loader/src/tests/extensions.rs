use std::path::PathBuf;

use crate::{
    extension::{ExtensionConfig, ExtensionLoader, WasmConfig},
    tests::create_shared_resources,
};
use engine_schema::Schema;
use extension_catalog::{ExtensionId, TypeDiscriminants};
use futures::{
    StreamExt, TryStreamExt,
    stream::{FuturesOrdered, FuturesUnordered},
};
use http::{HeaderMap, HeaderValue};
use runtime::extension::Token;
use serde_json::json;

const LATEST_SDK: semver::Version = semver::Version::new(0, 14, 0);

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

    let (shared, _) = create_shared_resources();

    let loader = ExtensionLoader::new(
        &Schema::from_sdl_or_panic("").await,
        shared,
        ExtensionConfig {
            id: ExtensionId::from(0usize),
            r#type: TypeDiscriminants::Authentication,
            manifest_id: "caching_auth-1.0.0".parse().unwrap(),
            sdk_version: LATEST_SDK,
            pool: Default::default(),
            wasm: config,
            guest_config: Some(json!({
                "cache_config": "test"
            })),
        },
    )
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("valid"));

    let (headers, token) = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(headers.into())
        .await
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
    let (shared, _) = create_shared_resources();

    let loader = ExtensionLoader::new(
        &Schema::from_sdl_or_panic("").await,
        shared,
        ExtensionConfig {
            id: ExtensionId::from(0usize),
            r#type: TypeDiscriminants::Authentication,
            manifest_id: "caching_auth-1.0.0".parse().unwrap(),
            sdk_version: LATEST_SDK,
            pool: Default::default(),
            wasm: config,
            guest_config: Some(json!({
                "cache_config": "test"
            })),
        },
    )
    .unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("valid"));

    let err = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(http::HeaderMap::new().into())
        .await
        .err();

    insta::assert_debug_snapshot!(err, @r#"
    Some(
        Guest(
            ErrorResponse {
                status-code: 401,
                errors: [],
            },
        ),
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
    let (shared, _) = create_shared_resources();

    let loader = ExtensionLoader::new(
        &Schema::from_sdl_or_panic("").await,
        shared,
        ExtensionConfig {
            id: ExtensionId::from(0usize),
            r#type: TypeDiscriminants::Authentication,
            manifest_id: "caching_auth-1.0.0".parse().unwrap(),
            sdk_version: LATEST_SDK,
            pool: Default::default(),
            wasm: config,
            guest_config: Some(json!({
                "cache_config": "test"
            })),
        },
    )
    .unwrap();

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

            let (_, token) = extension.authenticate(headers.into()).await.unwrap();
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
    let (_, token) = loader
        .instantiate()
        .await
        .unwrap()
        .authenticate(headers.into())
        .await
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

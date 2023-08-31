use std::borrow::Cow;

use grafbase_sdk::kv::{KvStore, ListOptionsBuilder, PutOptionsBuilder};
use serde::{Deserialize, Serialize};
use worker::{js_sys::JSON, wasm_bindgen::JsValue, Error::RustError, Request, Response, Result, RouteContext, Url};

pub async fn get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(key) = ctx.param("key") else {
        return Response::error("missing key", 400);
    };

    let kv_store = KvStore::new(&ctx.env)?;
    let js_value = kv_store.get(key, None).await?.unwrap_or(JsValue::from_str("not found"));

    Response::ok(js_value.as_string().unwrap())
}

pub async fn get_metadata(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(key) = ctx.param("key") else {
        return Response::error("missing key", 400);
    };

    let kv_store = KvStore::new(&ctx.env)?;
    let js_value = kv_store.get_with_metadata(key, None).await?;

    let Some(get_metadata) = js_value else {
        return Response::ok("not found");
    };

    #[derive(Debug, serde::Serialize)]
    struct ResponseBody {
        pub value: String,
        pub metadata: serde_json::Value,
    }

    let metadata: String = JSON::stringify(&get_metadata.metadata())?.into();

    let response_body = ResponseBody {
        value: get_metadata.value().as_string().unwrap(),
        metadata: serde_json::from_str(&metadata)?,
    };

    Response::from_json(&response_body)
}

pub async fn put(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(key) = ctx.param("key") else {
        return Response::error("missing key", 400);
    };

    let value = req.text().await?;

    let kv_store = KvStore::new(&ctx.env)?;
    kv_store.put(key, &JsValue::from(value), None).await?;

    Response::empty()
}

pub async fn put_metadata(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(key) = ctx.param("key") else {
        return Response::error("missing key", 400);
    };

    #[derive(Debug, serde::Deserialize)]
    struct RequestBody {
        value: String,
        metadata: serde_json::Value,
    }

    let body = req.json::<RequestBody>().await?;

    let metadata = serde_json::to_string(&body.metadata)?;
    let kv_store = KvStore::new(&ctx.env)?;
    let put_options = PutOptionsBuilder::default()
        .metadata(JsValue::from(metadata))
        .build()
        .map_err(|err| RustError(err.to_string()))?;

    kv_store.put(key, &JsValue::from(body.value), Some(put_options)).await?;

    Response::empty()
}

pub async fn delete(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(key) = ctx.param("key") else {
        return Response::error("missing key", 400);
    };

    let kv_store = KvStore::new(&ctx.env)?;
    kv_store.delete(key).await?;

    Response::empty()
}

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;

    let prefix = get_query_parameter(&url, "prefix").map(|str| str.to_string());
    let limit = get_query_parameter(&url, "limit")
        .map(|str| str.parse::<u32>())
        .transpose()
        .map_err(|err| RustError(err.to_string()))?;
    let cursor = get_query_parameter(&url, "cursor").map(|str| str.to_string());

    let list_options = ListOptionsBuilder::default()
        .prefix(prefix)
        .limit(limit)
        .cursor(cursor)
        .build()
        .map_err(|err| RustError(err.to_string()))?;

    let kv_store = KvStore::new(&ctx.env)?;
    let js_value = kv_store.list(Some(list_options)).await?;

    #[derive(Debug, Serialize, Deserialize)]
    struct ListKey {
        name: String,
        expiration: Option<u32>,
        metadata: Option<serde_json::Value>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ListResponse {
        cursor: Option<String>,
        list_complete: bool,
        keys: Vec<ListKey>,
    }

    let list_response = serde_wasm_bindgen::from_value::<ListResponse>(js_value)?;

    Response::from_json(&list_response)
}

fn get_query_parameter<'a>(url: &'a Url, parameter: &str) -> Option<Cow<'a, str>> {
    url.query_pairs().find(|(k, _)| k == parameter).map(|pair| pair.1)
}

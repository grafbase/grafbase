use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use worker::{
    js_sys::{JsString, Object, Reflect},
    wasm_bindgen::prelude::*,
    Error,
    Error::RustError,
    Result,
};

use crate::{api::js_object, sys::kv::KvStore as KvStoreSys};

const KV_BASE_PREFIX: &str = "KV_BASE_PREFIX";
const KV_ID: &str = "KV_ID";
const KV_MAX_LIST_SIZE: u32 = 1000;

#[wasm_bindgen]
pub struct KvStore {
    // this field is private intentionally. it shouldn't be accessible or mutable in JS
    base_prefix: String,

    inner: KvStoreSys,
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
pub enum KvError {
    #[error("Missing KV_ID env var")]
    MissingKvId,
    #[error("KV_BASE_PREFIX env var should not be empty")]
    EmptyBasePrefix,
}

impl From<KvError> for Error {
    fn from(value: KvError) -> Self {
        RustError(value.to_string())
    }
}

#[wasm_bindgen]
impl KvStore {
    #[wasm_bindgen(constructor)]
    pub fn new(env: &worker::Env) -> Result<KvStore> {
        use crate::ext::env::EnvExt;

        let base_prefix = env
            .var(KV_BASE_PREFIX)
            .map(|env_var| env_var.to_string())
            .unwrap_or_default();

        if base_prefix.is_empty() {
            return Err(KvError::EmptyBasePrefix.into());
        }

        let Ok(kv_id) = env.var(KV_ID) else {
            return Err(KvError::MissingKvId.into());
        };

        let kv_store = EnvExt::kv(env, &kv_id.to_string())?;

        Ok(Self {
            base_prefix,
            inner: kv_store,
        })
    }

    /// Gets a value
    pub async fn get(&self, key: &str, options: Option<GetOptions>) -> Result<Option<JsValue>> {
        let options = options.map(Object::from).unwrap_or_default();

        let promise = self.inner.get(&prefix_str(&self.base_prefix, key), options)?;

        JsFuture::from(promise)
            .await
            .map(|val| if exists(&val) { Some(val) } else { None })
            .map_err(Error::from)
    }

    /// Gets a value with metadata
    pub async fn get_with_metadata(
        &self,
        key: &str,
        options: Option<GetOptions>,
    ) -> Result<Option<GetMetadataResponse>> {
        let options = options.map(Object::from).unwrap_or_default();

        let promise = self
            .inner
            .get_with_metadata(&prefix_str(&self.base_prefix, key), options)?;

        JsFuture::from(promise)
            .await
            .map(|val| {
                let metadata = Reflect::get(&val, &JsValue::from_str("metadata")).ok()?;
                let value = Reflect::get(&val, &JsValue::from_str("value")).ok()?;

                if exists(&value) {
                    Some(GetMetadataResponse { metadata, value })
                } else {
                    None
                }
            })
            .map_err(Error::from)
    }

    /// Puts a value
    pub async fn put(&self, key: &str, value: &JsValue, options: Option<PutOptions>) -> Result<()> {
        let options = options.map(Object::from).unwrap_or_default();

        let promise = self.inner.put(&prefix_str(&self.base_prefix, key), value, options)?;

        JsFuture::from(promise).await.map(|_| ()).map_err(Error::from)
    }

    /// Deletes a value
    pub async fn delete(&self, key: &str) -> Result<()> {
        JsFuture::from(self.inner.delete(&prefix_str(&self.base_prefix, key))?)
            .await
            .map(|_| ())
            .map_err(Error::from)
    }

    pub async fn list(&self, options: Option<ListOptions>) -> Result<JsValue> {
        let options = options
            .and_then(|options| {
                ListOptionsBuilder::default()
                    .cursor(options.cursor)
                    .prefix(options.prefix.map(|prefix| prefix_str(&self.base_prefix, &prefix)))
                    .limit(options.limit.map(|l| std::cmp::max(l, KV_MAX_LIST_SIZE)))
                    .build()
                    .ok()
            })
            .map(Object::from)
            .unwrap_or_default();

        let promise = self.inner.list(options)?;

        Ok(JsFuture::from(promise).await?)
    }
}

#[derive(Debug, Default)]
#[wasm_bindgen]
pub struct GetMetadataResponse {
    metadata: JsValue,
    value: JsValue,
}

// this implementation is required because public fields in structs with `wasm_bindgen` require `Copy`
// JsValue does not implement `Copy` therefore we have to explicitly create getters for the fields
#[wasm_bindgen]
impl GetMetadataResponse {
    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        self.metadata.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> JsValue {
        self.value.clone()
    }
}

/// For simple values, it often makes sense to use the default "text" type which provides you with
/// your value as a string. For convenience, a "json" type is also specified which will convert a JSON
/// value into an object before returning it to you. For large values, you can use "stream" to request a
/// ReadableStream and "arrayBuffer" to request an ArrayBuffer for binary values.
///
/// For large values, the choice of type can have a noticeable effect on latency and CPU usage.
/// For reference, the types can be ordered from fastest to slowest as "stream", "arrayBuffer", "text", and "json".
#[derive(Debug, Copy, Clone, Default)]
#[wasm_bindgen]
pub enum GetValueType {
    #[default]
    /// A string (default)
    Text,
    /// An object decoded from a JSON string
    Json,
    /// An [ArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer) instance
    ArrayBuffer,
    /// A [ReadableStream](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)
    Stream,
}

impl From<GetValueType> for JsValue {
    fn from(value: GetValueType) -> Self {
        match value {
            GetValueType::Text => JsValue::from_str("text"),
            GetValueType::Json => JsValue::from_str("json"),
            GetValueType::ArrayBuffer => JsValue::from_str("arrayBuffer"),
            GetValueType::Stream => JsValue::from_str("stream"),
        }
    }
}

#[derive(Debug, Default, derive_builder::Builder)]
#[builder(default)]
#[wasm_bindgen]
pub struct GetOptions {
    /// The cacheTtl parameter must be an integer that is greater than or equal to 60, which is the default.
    /// It defines the length of time in seconds that a KV result is cached in the global network location
    /// that it is accessed from
    cache_ttl: Option<u32>,
    /// Type of the value
    value_type: Option<GetValueType>,
}

#[wasm_bindgen]
impl GetOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(cache_ttl: Option<u32>, value_type: Option<GetValueType>) -> Self {
        GetOptions { cache_ttl, value_type }
    }
}

impl From<GetOptions> for Object {
    fn from(value: GetOptions) -> Self {
        js_object! {
            "cacheTtl" => value.cache_ttl,
            "type" => value.value_type
        }
    }
}

///Write a value identified by a key. Use URL-encoding to use special characters (for example, :, !, %)
/// in the key name. Body should be the value to be stored along with JSON metadata to be associated
/// with the key/value pair. Existing values, expirations, and metadata will be overwritten.
/// If neither expiration nor expiration_ttl is specified, the key-value pair will never expire.
/// If both are set, expiration_ttl is used and expiration is ignored.
#[derive(Debug, Default, derive_builder::Builder)]
#[builder(default)]
#[wasm_bindgen]
pub struct PutOptions {
    /// Value will expire at specified time. Seconds since epoch.
    expiration: Option<u32>,
    /// Time to live of the value in seconds
    expiration_ttl: Option<u32>,
    /// Arbitrary JSON to be associated with a key/value pair.
    metadata: JsValue,
}

#[wasm_bindgen]
impl PutOptions {
    #[wasm_bindgen(constructor)]
    // the absence of Option in metadata is not an oversight: https://github.com/rustwasm/wasm-bindgen/issues/1906#issuecomment-564142189
    pub fn new(expiration: Option<u32>, expiration_ttl: Option<u32>, metadata: JsValue) -> Self {
        PutOptions {
            expiration,
            expiration_ttl,
            metadata,
        }
    }
}

impl From<PutOptions> for Object {
    fn from(value: PutOptions) -> Self {
        js_object! {
            "expiration" => value.expiration,
            "expirationTtl" => value.expiration_ttl,
            "metadata" => value.metadata
        }
    }
}

#[derive(Debug, Default, derive_builder::Builder)]
#[builder(default)]
#[wasm_bindgen]
pub struct ListOptions {
    /// String that represents a prefix you can use to filter all keys
    prefix: Option<String>,
    /// Maximum number of keys returned. The default is 1,000, which is the maximum. It is unlikely that you will want to change this default but it is included for completeness.
    limit: Option<u32>,
    /// Used for paginating responses
    cursor: Option<String>,
}

#[wasm_bindgen]
impl ListOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(prefix: Option<String>, limit: Option<u32>, cursor: Option<String>) -> Self {
        ListOptions { prefix, limit, cursor }
    }
}

impl From<ListOptions> for Object {
    fn from(value: ListOptions) -> Self {
        js_object! {
            "prefix" => value.prefix,
            "limit" => value.limit,
            "cursor" => value.cursor
        }
    }
}

fn prefix_str(prefix: &str, target: &str) -> String {
    format!("{}/{}", prefix, target.trim_start_matches('/'))
}

fn exists(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

#[cfg(test)]
mod test {
    use crate::api::kv::prefix_str;

    #[test]
    fn should_prefix_correctly() {
        let prefix = "prefix";
        let target = "/test";

        let result = prefix_str(prefix, target);
        let expected = format!("{prefix}{target}");

        assert_eq!(expected, result);
    }
}

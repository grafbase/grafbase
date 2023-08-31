use wasm_bindgen::prelude::*;
use worker::{js_sys, EnvBinding};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone)]
    pub type KvStore;

    #[wasm_bindgen(structural, method, catch, js_class=KvStore, js_name=get)]
    pub fn get(this: &KvStore, key: &str, options: js_sys::Object) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=KvStore, js_name=getWithMetadata)]
    pub fn get_with_metadata(this: &KvStore, key: &str, options: js_sys::Object) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=KvStore, js_name=put)]
    pub fn put(this: &KvStore, key: &str, value: &JsValue, options: js_sys::Object)
        -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=KvStore, js_name=delete)]
    pub fn delete(this: &KvStore, key: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(structural, method, catch, js_class=KvStore, js_name=list)]
    pub fn list(this: &KvStore, options: js_sys::Object) -> Result<js_sys::Promise, JsValue>;
}

impl EnvBinding for KvStore {
    const TYPE_NAME: &'static str = "KvNamespace";
}

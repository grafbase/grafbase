pub mod kv;

// handy macro from the folks at [worker-rs](https://github.com/cloudflare/workers-rs/blob/main/worker/src/r2/builder.rs#L404)
// ty
macro_rules! js_object {
    {$($key: expr => $value: expr),* $(,)?} => {{
        let obj = Object::new();
        $(
            {
                let res = Reflect::set(&obj, &JsString::from($key), &JsValue::from($value));
                debug_assert!(res.is_ok(), "setting properties should never fail on our dictionary objects");
            }
        )*
        obj
    }};
}
pub(crate) use js_object;

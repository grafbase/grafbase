use http::StatusCode;

pub trait ResponseExt: Sized {
    type JsonError;
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Result<T, Self::JsonError>;
    fn error_for_status(self) -> Result<Self, ErrorWithStatus>;
}

pub struct ErrorWithStatus {
    pub status_code: StatusCode,
}

// FIXME: Move to runtime-specific abstractions.
impl ResponseExt for http::Response<bytes::Bytes> {
    #[cfg(target_arch = "wasm32")]
    type JsonError = serde_wasm_bindgen::Error;
    #[cfg(not(target_arch = "wasm32"))]
    type JsonError = serde_json::Error;

    #[cfg(target_arch = "wasm32")]
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Result<T, Self::JsonError> {
        let js_value: JsValue = Uint8Array::from(customer_body.as_slice()).into();
        serde_wasm_bindgen::from_value(js_value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Result<T, Self::JsonError> {
        let bytes = self.into_body().to_vec();
        serde_json::from_slice(&bytes)
    }

    fn error_for_status(self) -> Result<Self, ErrorWithStatus> {
        let status_code = self.status();
        if status_code.is_client_error() || status_code.is_server_error() {
            Err(ErrorWithStatus { status_code })
        } else {
            Ok(self)
        }
    }
}

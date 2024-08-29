use std::borrow::Cow;

#[derive(Debug)]
pub struct GraphqlHttpResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: anyhow::Result<serde_json::Value>,
}

impl GraphqlHttpResponse {
    #[track_caller]
    pub fn into_body(self) -> serde_json::Value {
        self.body.expect("JSON parsing on body failed")
    }

    #[track_caller]
    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.into_body() {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    #[track_caller]
    pub fn deserialize_data<T: serde::de::DeserializeOwned>(self) -> T {
        serde_json::from_value(self.into_data()).expect("to be able to deserialize")
    }

    #[track_caller]
    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.body.as_ref().expect("JSON parsing on body failed")["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }
}

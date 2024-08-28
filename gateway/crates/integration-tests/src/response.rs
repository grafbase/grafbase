use std::borrow::Cow;

#[derive(Debug)]
pub struct GraphqlHttpResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: serde_json::Value,
}

impl GraphqlHttpResponse {
    pub fn into_body(self) -> serde_json::Value {
        self.body
    }

    #[track_caller]
    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.body {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    pub fn deserialize_data<T: serde::de::DeserializeOwned>(self) -> T {
        serde_json::from_value(self.into_data()).expect("to be able to deserialize")
    }

    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.body["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }
}

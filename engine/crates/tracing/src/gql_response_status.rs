static X_GRAFBASE_GQL_RESPONSE_STATUS: http::HeaderName =
    http::HeaderName::from_static("x-grafbase-graphql-response-status");

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum GraphqlResponseStatus {
    Success,
    /// Error happened during the execution of the query
    FieldError {
        count: u64,
        data_is_null: bool,
    },
    /// Bad request, failed before the execution and `data` field isn't present.
    RequestError {
        count: u64,
    },
}

impl GraphqlResponseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::FieldError { data_is_null, .. } => {
                if data_is_null {
                    "FIELD_ERROR_NULL_DATA"
                } else {
                    "FIELD_ERROR"
                }
            }
            Self::RequestError { .. } => "REQUEST_ERROR",
        }
    }
}

impl GraphqlResponseStatus {
    pub fn header_name() -> &'static http::HeaderName {
        &X_GRAFBASE_GQL_RESPONSE_STATUS
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    // Used to generate a status for a streaming response or a batch request.
    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (s @ Self::RequestError { .. }, _) => s,
            (_, s @ Self::RequestError { .. }) => s,
            (Self::Success, s @ Self::FieldError { .. }) => s,
            (s @ Self::FieldError { .. }, Self::Success) => s,
            (Self::FieldError { count, data_is_null }, Self::FieldError { count: extra_count, .. }) => {
                Self::FieldError {
                    count: count + extra_count,
                    data_is_null,
                }
            }
            (Self::Success, Self::Success) => Self::Success,
        }
    }

    fn to_header_value(self) -> http::HeaderValue {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&self).expect("valid json"))
            .try_into()
            .expect("valid header value")
    }

    fn from_header_value(value: &http::HeaderValue) -> Option<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        let bytes = URL_SAFE_NO_PAD.decode(value.as_bytes()).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}

impl headers::Header for GraphqlResponseStatus {
    fn name() -> &'static http::HeaderName {
        &X_GRAFBASE_GQL_RESPONSE_STATUS
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        values
            .next()
            .and_then(Self::from_header_value)
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(self.to_header_value()))
    }
}

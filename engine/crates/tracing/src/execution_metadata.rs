/// header name for execution metadata
pub static X_GRAFBASE_GRAPHQL_EXECUTION_METADATA: http::HeaderName =
    http::HeaderName::from_static("x-grafbase-execution-metadata");

/// Execution metadata
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct GraphqlExecutionMetadata {
    /// The operation id
    pub operation_id: Option<String>,
    /// The operation type
    pub operation_type: Option<String>,
    /// The operation name
    pub operation_name: Option<String>,
    /// Whether the operation has errors
    pub has_errors: Option<bool>,
}

impl GraphqlExecutionMetadata {
    /// Operation id
    pub fn operation_id(&self) -> &str {
        self.operation_id.as_deref().unwrap_or_default()
    }

    /// Operation type
    pub fn operation_type(&self) -> &str {
        self.operation_type.as_deref().unwrap_or_default()
    }

    /// Operation name
    pub fn operation_name(&self) -> &str {
        self.operation_name.as_deref().unwrap_or_default()
    }

    /// Whether the operation has errors
    pub fn has_errors(&self) -> bool {
        self.has_errors.unwrap_or_default()
    }
}

impl headers::Header for GraphqlExecutionMetadata {
    fn name() -> &'static http::HeaderName {
        &X_GRAFBASE_GRAPHQL_EXECUTION_METADATA
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        values
            .filter_map(|value| {
                let decoded = URL_SAFE_NO_PAD.decode(value.as_bytes()).ok()?;
                postcard::from_bytes(&decoded).ok()
            })
            .last()
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        let value = postcard::to_stdvec(self).unwrap();
        let encoded = URL_SAFE_NO_PAD.encode(value);
        values.extend(Some(http::HeaderValue::from_str(&encoded).unwrap()));
    }
}

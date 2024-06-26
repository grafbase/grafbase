use std::{collections::HashMap, sync::Arc};

use runtime::hooks::{HeaderMap, HookError, HooksImpl, UserError};
pub use wasi_component_loader::{ComponentLoader, Config as WasiConfig};

pub struct HooksWasi(ComponentLoader);

impl HooksWasi {
    pub fn new(loader: ComponentLoader) -> Self {
        Self(loader)
    }
}

#[async_trait::async_trait]
impl HooksImpl for HooksWasi {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), HookError> {
        let context = Self::Context::new();

        Ok(self
            .0
            .on_gateway_request(context, headers)
            .await
            .map_err(to_local_error)?)
    }

    async fn authorized(
        &self,
        context: Arc<Self::Context>,
        rule: String,
        input: Vec<String>,
    ) -> Result<Vec<Option<UserError>>, HookError> {
        let results = self
            .0
            .authorized(context, rule, input)
            .await
            .map_err(to_local_error)?
            .into_iter()
            .map(|result| result.map(error_response_to_user_error))
            .collect();

        Ok(results)
    }
}

fn to_local_error(error: wasi_component_loader::Error) -> HookError {
    match error {
        wasi_component_loader::Error::Internal(error) => HookError::Internal(error.into()),
        wasi_component_loader::Error::User(error) => HookError::User(error_response_to_user_error(error)),
    }
}

fn error_response_to_user_error(error: wasi_component_loader::ErrorResponse) -> UserError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key, value)
        })
        .collect();

    UserError {
        message: error.message,
        extensions,
    }
}

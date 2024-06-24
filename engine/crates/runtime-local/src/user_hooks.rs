use std::collections::HashMap;

use runtime::user_hooks::{HeaderMap, UserError, UserHookError, UserHooksImpl};
pub use wasi_component_loader::{ComponentLoader, Config as WasiConfig};

pub struct UserHooksWasi(ComponentLoader);

impl UserHooksWasi {
    pub fn new(loader: ComponentLoader) -> Self {
        Self(loader)
    }
}

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksWasi {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), UserHookError> {
        let context = Self::Context::new();

        Ok(self
            .0
            .on_gateway_request(context, headers)
            .await
            .map_err(to_local_error)?)
    }

    async fn on_authorization(
        &self,
        context: Self::Context,
        input: Vec<String>,
    ) -> Result<(Self::Context, Vec<Option<UserError>>), UserHookError> {
        let (context, results) = self.0.on_authorization(context, input).await.map_err(to_local_error)?;

        let results = results
            .into_iter()
            .map(|result| result.map(error_response_to_user_error))
            .collect();

        Ok((context, results))
    }
}

fn to_local_error(error: wasi_component_loader::Error) -> UserHookError {
    match error {
        wasi_component_loader::Error::Internal(error) => UserHookError::Internal(error.into()),
        wasi_component_loader::Error::User(error) => UserHookError::User(error_response_to_user_error(error)),
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

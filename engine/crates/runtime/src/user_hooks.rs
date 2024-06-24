use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub use http::HeaderMap;

#[derive(Debug, thiserror::Error)]
pub enum UserHookError {
    #[error("Kv error: {0}")]
    User(UserError),
    #[error("{0}")]
    Internal(Box<dyn std::error::Error>),
}

/// An error type available for the user to throw from the guest.
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub struct UserError {
    /// Optional extensions added to the response
    pub extensions: BTreeMap<String, serde_json::Value>,
    /// The error message
    pub message: String,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

#[derive(Clone)]
pub struct UserHooks(Arc<dyn UserHooksImpl<Context = HashMap<String, String>>>);

impl UserHooks {
    pub fn new(inner: impl UserHooksImpl<Context = HashMap<String, String>> + 'static) -> Self {
        Self(Arc::new(inner))
    }
}

impl std::ops::Deref for UserHooks {
    type Target = dyn UserHooksImpl<Context = HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[async_trait::async_trait]
pub trait UserHooksImpl: Send + Sync {
    type Context;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), UserHookError>;

    async fn on_authorization(
        &self,
        context: Self::Context,
        input: Vec<String>,
    ) -> Result<(Self::Context, Vec<Option<UserError>>), UserHookError>;
}

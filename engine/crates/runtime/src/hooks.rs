use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub use http::HeaderMap;

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Kv error: {0}")]
    User(UserError),
    #[error("{0}")]
    Internal(Box<dyn std::error::Error>),
}

impl From<&'static str> for HookError {
    fn from(err: &'static str) -> Self {
        HookError::Internal(err.into())
    }
}

impl HookError {
    pub fn is_user_error(&self) -> bool {
        matches!(self, HookError::User(_))
    }
}

/// An error type available for the user to throw from the guest.
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub struct UserError {
    /// Optional extensions added to the response
    pub extensions: BTreeMap<String, serde_json::Value>,
    /// The error message
    pub message: String,
}

impl From<&'static str> for UserError {
    fn from(message: &'static str) -> Self {
        UserError {
            extensions: BTreeMap::new(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

#[derive(Clone)]
pub struct Hooks(Arc<dyn HooksImpl<Context = HashMap<String, String>>>);

impl Hooks {
    pub fn new(inner: impl HooksImpl<Context = HashMap<String, String>> + 'static) -> Self {
        Self(Arc::new(inner))
    }
}

impl std::ops::Deref for Hooks {
    type Target = dyn HooksImpl<Context = HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[async_trait::async_trait]
pub trait HooksImpl: Send + Sync {
    type Context;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), HookError>;

    async fn authorized(
        &self,
        context: Arc<Self::Context>,
        rule: String,
        input: Vec<String>,
    ) -> Result<Vec<Option<UserError>>, HookError>;
}

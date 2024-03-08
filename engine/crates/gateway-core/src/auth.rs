use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum AdminAuthError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[async_trait::async_trait]
pub trait Authorizer: Send + Sync {
    type Context;
    async fn authorize_admin_request(
        &self,
        ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError>;
}

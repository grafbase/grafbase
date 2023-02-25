use crate::DynamoDBContext;
use std::sync::Arc;

pub struct CustomResolvers;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CustomResolversError {
    #[error("An internal error happened")]
    UnknownError,
}

impl CustomResolvers {
    pub async fn invoke(
        &self,
        _resolver_name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value, CustomResolversError> {
        Ok(serde_json::Value::String("Hello World".to_string()))
    }
}

pub fn get_custom_resolvers(_ctx: Arc<DynamoDBContext>) -> CustomResolvers {
    CustomResolvers
}

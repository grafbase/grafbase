use quick_error::quick_error;

pub struct CustomResolvers;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum CustomResolversError {
        UnknownError {
            display("An internal error happened")
        }
    }
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

pub fn get_custom_resolvers() -> CustomResolvers {
    CustomResolvers
}

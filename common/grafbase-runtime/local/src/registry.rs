use async_trait::async_trait;
use grafbase_runtime::registry::{RegistryError, RegistryProvider, RegistryResult};
use worker::Env;

pub struct Registry<'a> {
    env: &'a Env,
    var_name: &'a str,
}

impl<'a> Registry<'a> {
    pub fn new(env: &'a Env, var_name: &'a str) -> Self {
        Registry { env, var_name }
    }
}

#[async_trait(?Send)]
impl RegistryProvider<serde_json::Value> for Registry<'_> {
    async fn get_registry(&self) -> RegistryResult<serde_json::Value> {
        use worker_utils::{EnvExt, VarType};

        let registry: String = self
            .env
            .var_get(VarType::Var, self.var_name)
            .map_err(|_err| RegistryError::MissingEnvVar(self.var_name.to_string()))?;

        Ok(serde_json::from_str(&registry).unwrap())
    }
}

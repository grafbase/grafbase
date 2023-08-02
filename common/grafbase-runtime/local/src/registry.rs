use async_trait::async_trait;
use gateway_protocol::VersionedRegistry;
use registry_provider::{RegistryError, RegistryProvider, RegistryResult};
use worker::{wasm_bindgen::JsValue, Env};

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
impl RegistryProvider for Registry<'_> {
    async fn get_registry(&self) -> RegistryResult<VersionedRegistry<'static>> {
        use worker_utils::{EnvExt, VarType};
        let registry = self
            .env
            .js_var_get(VarType::Var, self.var_name)
            .map_err(|_err| RegistryError::MissingEnvVar(self.var_name.to_string()))?;

        let registry = Into::<JsValue>::into(registry)
            .as_string()
            .expect("Failed while parsing registry as String.");

        Ok(serde_json::from_str(&registry).expect("Couldn't parse the registry into a VersionedRegistry."))
    }
}

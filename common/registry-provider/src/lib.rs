use async_trait::async_trait;
use gateway_protocol::VersionedRegistry;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("{0}")]
    RemoteConfigError(String),
    #[error("Environment variable `{0}` not found")]
    MissingEnvVar(String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

#[async_trait(?Send)]
pub trait RegistryProvider {
    async fn get_registry(&self) -> RegistryResult<VersionedRegistry<'static>>;
}

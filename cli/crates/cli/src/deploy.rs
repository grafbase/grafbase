use crate::errors::CliError;
use backend::api::deploy;

/// # Errors
#[tokio::main]
pub async fn deploy() -> Result<(), CliError> {
    deploy::deploy().await.map_err(CliError::BackendApiError)
}

use crate::{errors::CliError, output::report};
use backend::api::deploy;

/// # Errors
#[tokio::main]
pub async fn deploy() -> Result<(), CliError> {
    report::deploy();
    deploy::deploy().await.map_err(CliError::BackendApiError)?;
    report::deploy_success();
    Ok(())
}

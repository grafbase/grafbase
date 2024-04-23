use backend::api::branch;

use crate::{cli_input::BranchRef, errors::CliError, output::report};

#[tokio::main]
pub async fn delete(branch_ref: BranchRef) -> Result<(), CliError> {
    report::delete_branch();

    branch::delete(branch_ref.account(), branch_ref.project(), branch_ref.branch())
        .await
        .map_err(CliError::BackendApiError)?;

    report::delete_branch_success();

    Ok(())
}

#[tokio::main]
pub async fn list() -> Result<(), CliError> {
    let branches = branch::list().await.map_err(CliError::BackendApiError)?;
    report::list_branches(branches);

    Ok(())
}

use std::time::Duration;

use crate::{create::create_impl, errors::CliError, link::link_impl, output::report, prompts::handle_inquire_error};
use backend::api::deploy;
use chrono::{DateTime, Local, Utc};
use common::{consts::PROJECT_METADATA_FILE, environment::Project};
use inquire::Select;
use strum::{Display, VariantArray};

#[derive(VariantArray, Display, Clone)]
pub enum UnlinkedDeploymentMethod {
    Link,
    Create,
}

#[tokio::main]
pub async fn deploy(branch: Option<&str>) -> Result<(), CliError> {
    let project = Project::get();

    let project_metadata_file_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

    match project_metadata_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => {
            let command_to_run = Select::new(
                "Your project does not appear to be linked. Would you like to create a new project or link to an existing one?",
                UnlinkedDeploymentMethod::VARIANTS.to_vec(),
            )
            .prompt()
            .map_err(handle_inquire_error)?;

            match command_to_run {
                UnlinkedDeploymentMethod::Link => {
                    link_impl(None).await?;
                    report::command_separator();
                }
                UnlinkedDeploymentMethod::Create => return create_impl(&None).await,
            }
        }
        Err(error) => return Err(CliError::ReadProjectMetadataFile(error)),
    }

    report::deploy();

    let deployment_id = deploy::deploy(branch).await.map_err(CliError::BackendApiError)?;
    report_progress(deployment_id.into_inner()).await?;

    report::deploy_success();

    Ok(())
}

async fn report_progress(deployment_id: String) -> Result<(), CliError> {
    const WAIT_DURATION: Duration = Duration::from_secs(10);
    const POLL_TIMEOUT: Duration = Duration::from_secs(1);

    let mut finished = false;
    let mut interval = tokio::time::interval(POLL_TIMEOUT);
    let mut last_index = 0;
    let mut failed = false;

    while !finished {
        interval.tick().await;

        let deployment = deploy::fetch_logs(deployment_id.clone().into())
            .await
            .map_err(CliError::BackendApiError)?;

        let mut seen_index = 0;

        for (i, log) in deployment.log_entries.iter().enumerate() {
            seen_index = i;

            if i <= last_index {
                continue;
            }

            let created_at: DateTime<Local> = log.created_at.into();
            println!("{}    {}", created_at.format("%H:%M:%S%.3f"), log.message);
        }

        last_index = seen_index;

        if let Some(finished_at) = deployment.finished_at {
            failed = deployment.status.failed();
            finished = (Utc::now() - finished_at).to_std().unwrap_or_default() > WAIT_DURATION;
        }
    }

    if failed {
        Err(CliError::DeploymentFailed)
    } else {
        Ok(())
    }
}

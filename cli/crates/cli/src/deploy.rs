use crate::{create::create_impl, errors::CliError, link::link_impl, output::report, prompts::handle_inquire_error};
use backend::api::{consts::PROJECT_METADATA_FILE, deploy};
use common::environment::Project;
use inquire::Select;
use strum::{Display, VariantArray};

#[derive(VariantArray, Display, Clone)]
enum UnlinkedDeploymentMethod {
    Link,
    Create,
}

#[tokio::main]
pub async fn deploy() -> Result<(), CliError> {
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
    deploy::deploy().await.map_err(CliError::BackendApiError)?;
    report::deploy_success();

    Ok(())
}

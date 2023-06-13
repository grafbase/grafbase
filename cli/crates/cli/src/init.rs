use crate::{cli_input::ConfigFormat, errors::CliError, output::report, prompts::handle_inquire_error};
use backend::project::{self, ConfigType, ProjectTemplate};
use inquire::Select;

pub fn init(name: Option<&str>, template: Option<&str>, config_format: Option<ConfigFormat>) -> Result<(), CliError> {
    let template = match (template, config_format) {
        (Some(template), _) => ProjectTemplate::FromUrl(template),
        (None, Some(ConfigFormat::TypeScript)) => ProjectTemplate::FromDefault(ConfigType::TypeScript),
        (None, Some(ConfigFormat::GraphQL)) => ProjectTemplate::FromDefault(ConfigType::GraphQL),
        (None, None) => {
            let config_type = Select::new(
                "In which format the project schema and configuration should be written?",
                ConfigType::VARIANTS.to_vec(),
            )
            .prompt()
            .map_err(handle_inquire_error)?;

            ProjectTemplate::FromDefault(config_type)
        }
    };

    project::init(name, template).map_err(CliError::BackendError)?;
    report::project_created(name, template);

    Ok(())
}

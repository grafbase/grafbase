use crate::{cli_input::ConfigFormat, errors::CliError, output::report, prompts::handle_inquire_error};
use backend::project::{self, ConfigType, Template};
use inquire::Select;

pub fn init(name: Option<&str>, template: Option<&str>, config_format: Option<ConfigFormat>) -> Result<(), CliError> {
    let template = match (template, config_format) {
        (Some(template), _) => Template::FromUrl(template),
        (None, Some(ConfigFormat::TypeScript)) => Template::FromDefault(ConfigType::TypeScript),
        (None, Some(ConfigFormat::GraphQL)) => Template::FromDefault(ConfigType::GraphQL),
        (None, None) => {
            let config_type = Select::new(
                "What configuration format would you like to use?",
                ConfigType::VARIANTS.to_vec(),
            )
            .prompt()
            .map_err(handle_inquire_error)?;

            Template::FromDefault(config_type)
        }
    };

    let config_type = project::init(name, template).map_err(CliError::BackendError)?;
    report::project_created(name, config_type);

    Ok(())
}

use crate::{
    cli_input::ConfigFormat, errors::CliError, output::report, prompts::handle_inquire_error, watercolor::watercolor,
};
use backend::{
    errors::BackendError,
    project::{self, ConfigType, Template},
};
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

    match project::init(name, template) {
        Ok(_) => report::project_created(name, template),
        Err(BackendError::NpmNotFound(_)) => {
            report::project_created(name, template);

            println!(
                "We've added our SDK to your {}, make sure to install dependencies before continuing.",
                watercolor!("package.json", @BrightBlue)
            );
        }
        Err(e) => return Err(CliError::BackendError(e)),
    }

    Ok(())
}

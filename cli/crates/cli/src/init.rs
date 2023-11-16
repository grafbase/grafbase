use crate::{cli_input::GraphType, errors::CliError, output::report, prompts::handle_inquire_error};
use backend::project::{self, Template};
use inquire::Select;

pub fn init(name: Option<&str>, template: Option<&str>, graph_type: Option<GraphType>) -> Result<(), CliError> {
    let template = match (template, graph_type) {
        (Some(template), _) => Template::FromUrl(template),
        (None, Some(GraphType::Single)) => Template::FromDefault(project::GraphType::Single),
        (None, Some(GraphType::Federated)) => Template::FromDefault(project::GraphType::Federated),
        (None, None) => {
            let graph_type = Select::new(
                "What type of graph would you like to create?",
                project::GraphType::VARIANTS.to_vec(),
            )
            .prompt()
            .map_err(handle_inquire_error)?;

            Template::FromDefault(graph_type)
        }
    };

    project::init(name, template).map_err(CliError::BackendError)?;
    report::project_created(name);

    Ok(())
}

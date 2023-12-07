use crate::{cli_input::IntrospectCommand, errors::CliError, output::report};
use tokio::runtime::Runtime;

pub(crate) fn introspect(command: &IntrospectCommand) -> Result<(), CliError> {
    match (command.url(), command.dev) {
        (Some(url), _) => {
            let headers = command.headers().collect::<Vec<_>>();
            introspect_remote(url, &headers)
        }
        (None, true) => introspect_local(),
        (None, false) => {
            eprintln!("Error: Either the --url or the --dev argument must be provided.");
            std::process::exit(1);
        }
    }
}

fn introspect_local() -> Result<(), CliError> {
    match server::introspect_local().map_err(CliError::ServerError)? {
        server::IntrospectLocalOutput::Sdl(schema) => {
            println!("{schema}");
        }
        server::IntrospectLocalOutput::EmptyFederated => {
            report::federated_schema_local_introspection_not_implemented();
        }
    }

    Ok(())
}

fn introspect_remote(url: &str, headers: &[(&str, &str)]) -> Result<(), CliError> {
    let operation = graphql_introspection::introspect(url, headers);

    match Runtime::new().unwrap().block_on(operation) {
        Ok(result) => Ok(println!("{result}")),
        Err(e) => Err(CliError::Introspection(e)),
    }
}

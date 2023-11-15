use crate::{cli_input::IntrospectCommand, errors::CliError};
use tokio::runtime::Runtime;

pub(crate) fn introspect(command: &IntrospectCommand) -> Result<(), CliError> {
    let headers = command.headers().collect::<Vec<_>>();
    let operation = graphql_introspection::introspect(command.url(), &headers);

    match Runtime::new().unwrap().block_on(operation) {
        Ok(result) => Ok(println!("{result}")),
        Err(e) => Err(CliError::Introspection(e)),
    }
}

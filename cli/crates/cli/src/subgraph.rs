use tokio::runtime::Runtime;

use crate::{
    cli_input::{SubgraphCommand, SubgraphCommandKind},
    errors::CliError,
};

pub(super) fn subgraph(cmd: SubgraphCommand) -> Result<(), CliError> {
    match cmd.kind {
        SubgraphCommandKind::Introspect(command) => {
            let headers = command.headers().collect::<Vec<_>>();
            let operation = graphql_introspection::introspect(command.url(), &headers);

            match Runtime::new().unwrap().block_on(operation) {
                Ok(result) => println!("{result}"),
                Err(e) => return Err(CliError::Introspection(e)),
            }
        }
    }

    Ok(())
}

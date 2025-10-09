use crate::{
    api,
    cli_input::{
        SchemaProposalCommand, SchemaProposalCreateCommand, SchemaProposalEditCommand, SchemaProposalSchemaSource,
        SchemaProposalSubCommand,
    },
    errors::CliError,
    output::report,
};
use std::{
    fs,
    io::{self, IsTerminal, Read},
};

#[tokio::main]
pub(crate) async fn schema_proposal(command: SchemaProposalCommand) -> Result<(), CliError> {
    match command.command {
        SchemaProposalSubCommand::Create(cmd) => handle_create(cmd).await,
        SchemaProposalSubCommand::Edit(cmd) => handle_edit(cmd).await,
    }
}

async fn handle_create(cmd: SchemaProposalCreateCommand) -> Result<(), CliError> {
    let SchemaProposalCreateCommand {
        graph_ref,
        name,
        description,
        subgraph_name,
        schema_source,
    } = cmd;

    let schema_input_provided = schema_source.is_provided();

    match (&subgraph_name, schema_input_provided) {
        (Some(_), false) => return Err(CliError::MissingArgument("--schema or --schema-stdin")),
        (None, true) => return Err(CliError::MissingArgument("--subgraph-name")),
        _ => {}
    }

    let branch = graph_ref.branch().ok_or(CliError::MissingArgument(
        "branch in graph reference (org/graph@branch)",
    ))?;

    let proposal_id = api::schema_proposal::create(
        graph_ref.account(),
        graph_ref.graph(),
        branch,
        &name,
        description.as_deref(),
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::schema_proposal_create_success(&proposal_id);

    if let Some(subgraph_name) = subgraph_name {
        let schema = read_schema(&schema_source)?;
        let subgraph = api::schema_proposal::SchemaEditSubgraph {
            name: &subgraph_name,
            schema: Some(schema.as_str()),
        };
        let subgraphs = [subgraph];

        if let Err(error) = apply_edit(&proposal_id, None, &subgraphs).await {
            report::schema_proposal_edit_after_create_failed(&proposal_id);
            return Err(error);
        }
    }

    Ok(())
}

async fn handle_edit(cmd: SchemaProposalEditCommand) -> Result<(), CliError> {
    let SchemaProposalEditCommand {
        schema_proposal_id,
        subgraph_name,
        description,
        schema_source,
    } = cmd;

    let schema = read_schema(&schema_source)?;

    let subgraph = api::schema_proposal::SchemaEditSubgraph {
        name: &subgraph_name,
        schema: Some(schema.as_str()),
    };

    let subgraphs = [subgraph];

    apply_edit(&schema_proposal_id, description.as_deref(), &subgraphs).await
}

fn read_schema(source: &SchemaProposalSchemaSource) -> Result<String, CliError> {
    if source.schema_stdin && source.schema_file_path.is_some() {
        return Err(CliError::MissingArgument("use either --schema or --schema-stdin"));
    }

    if let Some(path) = &source.schema_file_path {
        return fs::read_to_string(path).map_err(CliError::SchemaReadError);
    }

    if source.schema_stdin {
        if io::stdin().is_terminal() {
            return Err(CliError::MissingArgument("schema piped through stdin"));
        }

        let mut schema = String::new();
        io::stdin()
            .read_to_string(&mut schema)
            .map_err(CliError::SchemaReadError)?;

        return Ok(schema);
    }

    // Clap should ensure we never hit this, but be defensive.
    Err(CliError::MissingArgument("--schema or --schema-stdin"))
}

async fn apply_edit<'a>(
    schema_proposal_id: &str,
    description: Option<&str>,
    subgraphs: &[api::schema_proposal::SchemaEditSubgraph<'a>],
) -> Result<(), CliError> {
    match api::schema_proposal::edit(schema_proposal_id, description, subgraphs).await {
        Ok(()) => {
            report::schema_proposal_edit_success(schema_proposal_id);
            Ok(())
        }
        Err(api::errors::ApiError::SchemaProposalError(api::errors::SchemaProposalError::EditParserErrors {
            errors,
        })) => {
            report::schema_proposal_edit_parser_errors(&errors);
            Err(CliError::BackendApiError(api::errors::ApiError::SchemaProposalError(
                api::errors::SchemaProposalError::EditParserErrors { errors },
            )))
        }
        Err(error) => Err(CliError::BackendApiError(error)),
    }
}

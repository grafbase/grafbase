use super::FullGraphRef;
use clap::{Args, Parser};
use std::path::PathBuf;

/// Manage schema proposals
#[derive(Debug, Parser)]
pub(crate) struct SchemaProposalCommand {
    #[command(subcommand)]
    pub command: SchemaProposalSubCommand,
}

/// Manage schema proposals
#[derive(Debug, Parser)]
pub(crate) enum SchemaProposalSubCommand {
    Create(SchemaProposalCreateCommand),
    Edit(SchemaProposalEditCommand),
}

/// Create a new schema proposal and print the proposal ID to stdout.
#[derive(Debug, Parser)]
pub(crate) struct SchemaProposalCreateCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    /// Optionally create an initial revision by providing a subgraph name and SDL source.
    #[arg(long = "subgraph-name")]
    pub subgraph_name: Option<String>,
    #[command(flatten)]
    pub schema_source: SchemaProposalSchemaSource,
}

#[derive(Debug, Parser)]
pub(crate) struct SchemaProposalEditCommand {
    #[arg(long)]
    pub schema_proposal_id: String,
    #[arg(long)]
    pub subgraph_name: String,
    /// Optional revision message for the edit.
    #[arg(long)]
    pub description: Option<String>,
    #[command(flatten)]
    pub schema_source: SchemaProposalSchemaSource,
}

#[derive(Debug, Args)]
pub(crate) struct SchemaProposalSchemaSource {
    /// Read the schema contents from stdin.
    #[arg(long)]
    pub schema_stdin: bool,
    /// Read the schema contents from this file path.
    #[arg(long = "schema")]
    pub schema_file_path: Option<PathBuf>,
}

impl SchemaProposalSchemaSource {
    pub fn is_provided(&self) -> bool {
        self.schema_stdin || self.schema_file_path.is_some()
    }
}

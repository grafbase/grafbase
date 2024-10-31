mod branch;
mod branch_ref;
mod check;
mod completions;
mod create;
mod federated_graph;
mod graph_ref;
mod graph_ref_no_branch;
mod introspect;
mod lint;
mod login;
mod publish;
mod schema;
mod sub_command;
mod subgraphs;
mod trust;

pub(crate) use self::{check::CheckCommand, trust::TrustCommand};
pub(crate) use branch::BranchSubCommand;
pub(crate) use branch_ref::BranchRef;
pub(crate) use completions::CompletionsCommand;
pub(crate) use create::CreateCommand;
pub(crate) use graph_ref::FullGraphRef;
pub(crate) use introspect::IntrospectCommand;
pub(crate) use lint::LintCommand;
pub(crate) use login::LoginCommand;
pub(crate) use publish::PublishCommand;
pub(crate) use schema::SchemaCommand;
pub(crate) use sub_command::RequiresLogin;
pub(crate) use sub_command::SubCommand;
pub(crate) use subgraphs::SubgraphsCommand;

use clap::Parser;
use common::consts::TRACE_LOG_FILTER;
use std::path::PathBuf;

fn split_header(header: &str) -> Option<(&str, &str)> {
    header.find(':').map(|split_index| {
        let key = header[0..split_index].trim();
        let value = header[split_index + 1..].trim();

        (key, value)
    })
}

#[derive(Debug, Parser)]
#[command(name = "Grafbase CLI", version)]
/// The Grafbase command line interface
pub struct Args {
    /// Set the tracing level
    #[arg(short, long, default_value_t = 0)]
    pub trace: u16,
    #[arg(long, hide = true)]
    pub custom_trace_filter: Option<String>,
    #[command(subcommand)]
    pub command: SubCommand,
    /// An optional replacement path for the home directory
    #[arg(long)]
    pub home: Option<PathBuf>,
}

impl Args {
    pub fn log_filter(&self) -> Option<&str> {
        if let Some(custom_trace) = &self.custom_trace_filter {
            Some(custom_trace.as_str())
        } else if self.trace >= 1 {
            Some(TRACE_LOG_FILTER)
        } else {
            None
        }
    }
}

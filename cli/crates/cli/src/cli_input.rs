mod argument_names;
mod branch;
mod branch_ref;
mod build;
mod check;
mod completions;
mod create;
mod deploy;
mod dev;
mod environment;
mod federated_graph;
mod graph_ref_no_branch;
mod init;
mod introspect;
mod link;
mod lint;
mod log_level_filter;
mod logs;
mod project_ref;
mod publish;
mod schema;
mod start;
mod sub_command;
mod subgraphs;
mod trust;

pub(crate) use self::{check::CheckCommand, trust::TrustCommand};
pub(crate) use argument_names::{filter_existing_arguments, ArgumentNames};
pub(crate) use branch::BranchSubCommand;
pub(crate) use branch_ref::BranchRef;
pub(crate) use build::BuildCommand;
pub(crate) use completions::CompletionsCommand;
pub(crate) use create::CreateCommand;
pub(crate) use deploy::DeployCommand;
pub(crate) use dev::DevCommand;
pub(crate) use environment::{EnvironmentCommand, EnvironmentSubCommand};
pub(crate) use graph_ref_no_branch::GraphRefNoBranch;
pub(crate) use init::{GraphType, InitCommand};
pub(crate) use introspect::IntrospectCommand;
pub(crate) use link::LinkCommand;
pub(crate) use lint::LintCommand;
pub(crate) use log_level_filter::{LogLevelFilter, LogLevelFilters};
pub(crate) use logs::LogsCommand;
pub(crate) use project_ref::{ProjectRef, ProjectRefOrGraphRef};
pub(crate) use publish::PublishCommand;
pub(crate) use schema::SchemaCommand;
pub(crate) use start::StartCommand;
pub(crate) use sub_command::SubCommand;
pub(crate) use subgraphs::SubgraphsCommand;

use clap::Parser;
use common::consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER};
use std::path::PathBuf;

const DEFAULT_SUBGRAPH_PORT: u16 = 4000;

fn split_header(header: &str) -> Option<(&str, &str)> {
    header.find(':').map(|split_index| {
        let key = header[0..split_index].trim();
        let value = header[split_index + 1..].trim();

        (key, value)
    })
}

fn split_env_var(env_var: &str) -> Option<(&str, &str)> {
    env_var.find('=').map(|split_index| {
        let key = env_var[0..split_index].trim();
        let value = env_var[split_index + 1..].trim();

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
    #[command(subcommand)]
    pub command: SubCommand,
    /// An optional replacement path for the home directory
    #[arg(long)]
    pub home: Option<PathBuf>,
}

impl Args {
    pub fn log_filter(&self) -> &str {
        if self.trace >= 1 {
            TRACE_LOG_FILTER
        } else {
            DEFAULT_LOG_FILTER
        }
    }
}

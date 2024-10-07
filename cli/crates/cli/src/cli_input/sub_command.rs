use clap::Parser;

use crate::{cli_input::BranchSubCommand, create::GraphMode, is_not_direct_install};

use super::{
    branch::BranchCommand, trust::TrustCommand, ArgumentNames, CheckCommand, CompletionsCommand, CreateCommand,
    DevCommand, IntrospectCommand, LinkCommand, LintCommand, PublishCommand, SchemaCommand, SubgraphsCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubCommand {
    /// Graph branch management
    Branch(BranchCommand),
    /// Start the Grafbase local development server
    Dev(DevCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new graph
    Create(CreateCommand),
    /// Connect a local graph to a remote graph
    Link(LinkCommand),
    /// Disconnect a local graph from a remote graph
    Unlink,
    /// Introspect a graph and print its schema
    Introspect(IntrospectCommand),
    /// List subgraphs
    Subgraphs(SubgraphsCommand),
    /// Fetch a federated graph or a subgraph
    Schema(SchemaCommand),
    /// Publish a subgraph schema
    Publish(PublishCommand),
    /// Dump the registry as JSON.
    #[clap(hide = true)]
    DumpConfig,
    /// Check a graph for validation, composition and breaking change errors
    Check(CheckCommand),
    /// Submit a trusted documents manifest
    Trust(TrustCommand),
    /// Upgrade the installed version of the Grafbase CLI
    #[clap(hide=is_not_direct_install())]
    Upgrade,
    /// Lint a GraphQL schema
    Lint(LintCommand),
}

impl SubCommand {
    pub(crate) fn in_project_context(&self) -> bool {
        matches!(
            self,
            Self::Create(CreateCommand {
                mode: Some(GraphMode::Managed) | None,
                ..
            }) | Self::Branch(BranchCommand {
                command: BranchSubCommand::List,
            }) | Self::Dev(DevCommand { .. })
                | Self::Link(_)
                | Self::Unlink
                | Self::DumpConfig
                | Self::Introspect(IntrospectCommand {
                    dev: true,
                    url: None,
                    ..
                })
        )
    }
}

impl ArgumentNames for SubCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        match self {
            SubCommand::Dev(command) => command.argument_names(),
            SubCommand::Create(command) => command.argument_names(),
            SubCommand::Schema(_)
            | SubCommand::Branch(_)
            | SubCommand::Publish(_)
            | SubCommand::Check(_)
            | SubCommand::Subgraphs(_)
            | SubCommand::Introspect(_)
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Link(_)
            | SubCommand::Unlink
            | SubCommand::Completions(_)
            | SubCommand::DumpConfig
            | SubCommand::Trust(_)
            | SubCommand::Upgrade
            | SubCommand::Lint(_) => None,
        }
    }
}

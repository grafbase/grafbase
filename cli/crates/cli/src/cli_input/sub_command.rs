use clap::Parser;

use crate::{
    cli_input::{
        environment::{
            EnvironmentVariableCreateCommand, EnvironmentVariableDeleteCommand, EnvironmentVariableListCommand,
        },
        BranchSubCommand, EnvironmentSubCommand,
    },
    create::GraphMode,
    is_not_direct_install,
};

use super::{
    branch::BranchCommand, trust::TrustCommand, ArgumentNames, BuildCommand, CheckCommand, CompletionsCommand,
    CreateCommand, DeployCommand, DevCommand, EnvironmentCommand, InitCommand, IntrospectCommand, LinkCommand,
    LintCommand, LogsCommand, PublishCommand, SchemaCommand, StartCommand, SubgraphsCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubCommand {
    /// Graph branch management
    Branch(BranchCommand),
    /// Start the Grafbase local development server
    Dev(DevCommand),
    /// Modify graph environment variables
    #[clap(visible_alias = "env")]
    Environment(EnvironmentCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Sets up the current or a new project for Grafbase
    Init(InitCommand),
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new graph
    Create(CreateCommand),
    /// Deploy your project
    Deploy(DeployCommand),
    /// Connect a local graph to a remote graph
    Link(LinkCommand),
    /// Disconnect a local graph from a remote graph
    Unlink,
    /// Tail logs from a standalone graph
    Logs(LogsCommand),
    /// Start Grafbase in self-hosted mode
    Start(StartCommand),
    /// Build the Grafbase project in advance to avoid the resolver build step in the start
    /// command.
    Build(BuildCommand),
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
            }) | Self::Environment(EnvironmentCommand {
                command: EnvironmentSubCommand::List(EnvironmentVariableListCommand { graph_ref: None })
            }) | Self::Environment(EnvironmentCommand {
                command: EnvironmentSubCommand::Create(EnvironmentVariableCreateCommand { graph_ref: None, .. })
            }) | Self::Environment(EnvironmentCommand {
                command: EnvironmentSubCommand::Delete(EnvironmentVariableDeleteCommand { graph_ref: None, .. })
            }) | Self::Deploy(_)
                | Self::Dev(DevCommand { .. })
                | Self::Link(_)
                | Self::Logs(LogsCommand {
                    project_branch: None,
                    ..
                })
                | Self::Start(_)
                | Self::Build(_)
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
            SubCommand::Init(command) => command.argument_names(),
            SubCommand::Create(command) => command.argument_names(),
            SubCommand::Schema(_)
            | SubCommand::Branch(_)
            | SubCommand::Environment(_)
            | SubCommand::Publish(_)
            | SubCommand::Check(_)
            | SubCommand::Subgraphs(_)
            | SubCommand::Introspect(_)
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Deploy(_)
            | SubCommand::Link(_)
            | SubCommand::Unlink
            | SubCommand::Start(_)
            | SubCommand::Build(_)
            | SubCommand::Completions(_)
            | SubCommand::DumpConfig
            | SubCommand::Trust(_)
            | SubCommand::Upgrade
            | SubCommand::Lint(_)
            | SubCommand::Logs(_) => None,
        }
    }
}

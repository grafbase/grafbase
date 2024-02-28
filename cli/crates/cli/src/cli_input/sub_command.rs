use clap::Parser;

use super::{
    trust::TrustCommand, ArgumentNames, BuildCommand, CheckCommand, CompletionsCommand, CreateCommand, DevCommand,
    InitCommand, IntrospectCommand, LinkCommand, LogsCommand, PublishCommand, SchemaCommand, StartCommand,
    SubgraphsCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubCommand {
    /// Start the Grafbase local development server
    Dev(DevCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Sets up the current or a new project for Grafbase
    Init(InitCommand),
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new project
    Create(CreateCommand),
    /// Deploy your project
    Deploy,
    /// Connect a local project to a remote project
    Link(LinkCommand),
    /// Disconnect a local project from a remote project
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
    #[clap(hide = true)]
    Trust(TrustCommand),
}

impl SubCommand {
    pub(crate) fn in_project_context(&self) -> bool {
        matches!(
            self,
            Self::Create(_)
                | Self::Deploy
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
            | SubCommand::Publish(_)
            | SubCommand::Check(_)
            | SubCommand::Subgraphs(_)
            | SubCommand::Introspect(_)
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Deploy
            | SubCommand::Link(_)
            | SubCommand::Unlink
            | SubCommand::Start(_)
            | SubCommand::Build(_)
            | SubCommand::Completions(_)
            | SubCommand::DumpConfig
            | SubCommand::Trust(_)
            | SubCommand::Logs(_) => None,
        }
    }
}

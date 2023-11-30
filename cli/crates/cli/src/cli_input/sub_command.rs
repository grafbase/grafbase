use clap::Parser;

use super::{
    ArgumentNames, BuildCommand, CheckCommand, CompletionsCommand, CreateCommand, DevCommand, InitCommand,
    IntrospectCommand, LinkCommand, LogsCommand, PublishCommand, SchemaCommand, StartCommand, SubgraphsCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubCommand {
    /// Run your Grafbase project locally
    Dev(DevCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Sets up the current or a new project for Grafbase
    Init(InitCommand),
    /// Resets the local database for the current project
    Reset,
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
    /// Tails logs from a remote project
    Logs(LogsCommand),
    /// Run your Grafbase project locally in production mode
    Start(StartCommand),
    /// Build the Grafbase project in advance to avoid the resolver build step in the start
    /// command.
    Build(BuildCommand),
    /// Introspect a subgraph endpoint and print its schema
    Introspect(IntrospectCommand),
    /// List subgraphs
    #[clap(hide = true)]
    Subgraphs(SubgraphsCommand),
    /// Fetch a federated graph or a subgraph
    #[clap(hide = true)]
    Schema(SchemaCommand),
    /// Publish a subgraph to a federated graph
    #[clap(hide = true)]
    Publish(PublishCommand),
    /// Dump the registry as JSON.
    #[clap(hide = true)]
    DumpConfig,
    /// Check a graph or a subgraph for validation, composition and breaking change errors.
    #[clap(hide = true)]
    Check(CheckCommand),
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
                | Self::Reset
                | Self::Start(_)
                | Self::Build(_)
                | Self::Unlink
                | Self::DumpConfig
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
            | SubCommand::Reset
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Deploy
            | SubCommand::Link(_)
            | SubCommand::Unlink
            | SubCommand::Start(_)
            | SubCommand::Build(_)
            | SubCommand::Completions(_)
            | SubCommand::DumpConfig
            | SubCommand::Logs(_) => None,
        }
    }
}

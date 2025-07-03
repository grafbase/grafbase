use clap::Parser;

use crate::{cli_input::ExtensionSubCommand, is_not_direct_install};

use super::{
    CheckCommand, CompletionsCommand, CreateCommand, DevCommand, ExtensionCommand, IntrospectCommand, LintCommand,
    LoginCommand, PublishCommand, SchemaCommand, SubgraphCommand, branch::BranchCommand, compose::ComposeCommand,
    mcp::McpCommand, trust::TrustCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum SubCommand {
    /// Manage branches
    Branch(BranchCommand),
    /// Output completions for the chosen shell
    Completions(CompletionsCommand),
    /// List installed plugins
    Plugins,
    /// Login to your Grafbase account
    Login(LoginCommand),
    /// Logout from your Grafbase account
    Logout,
    /// Create a graph
    Create(CreateCommand),
    /// Compose a graph from subgraph schemas
    Compose(ComposeCommand),
    /// Introspect a schema
    Introspect(IntrospectCommand),
    /// Manage subgraphs
    Subgraph(SubgraphCommand),
    /// Fetch a schema from the registry
    Schema(SchemaCommand),
    /// Publish a schema to the registry
    Publish(PublishCommand),
    /// Run validation, composition and breaking change checks
    Check(CheckCommand),
    /// Submit a trusted documents manifest
    Trust(TrustCommand),
    /// Upgrade the Grafbase CLI
    #[clap(hide=is_not_direct_install())]
    Upgrade,
    /// Lint a schema
    Lint(LintCommand),
    /// Start the development server
    Dev(DevCommand),
    /// Start the MCP server
    Mcp(McpCommand),
    /// Manage extensions
    Extension(ExtensionCommand),
    #[clap(external_subcommand)]
    Plugin(Vec<String>),
}

pub trait RequiresLogin {
    fn requires_login(&self) -> bool;
}

impl RequiresLogin for SubCommand {
    fn requires_login(&self) -> bool {
        matches!(
            self,
            SubCommand::Create(_)
                | SubCommand::Publish(_)
                | SubCommand::Trust(_)
                | SubCommand::Subgraph(_)
                | SubCommand::Check(_)
                | SubCommand::Branch(_)
                | SubCommand::Schema(_)
                | SubCommand::Compose(ComposeCommand { graph_ref: Some(_), .. })
                | SubCommand::Dev(DevCommand { graph_ref: Some(_), .. })
                | SubCommand::Extension(ExtensionCommand {
                    command: ExtensionSubCommand::Publish(_)
                })
        )
    }
}

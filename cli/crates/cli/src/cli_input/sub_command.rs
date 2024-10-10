use clap::Parser;

use crate::is_not_direct_install;

use super::{
    branch::BranchCommand, trust::TrustCommand, ArgumentNames, CheckCommand, CompletionsCommand, CreateCommand,
    IntrospectCommand, LintCommand, PublishCommand, SchemaCommand, SubgraphsCommand,
};

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubCommand {
    /// Graph branch management
    Branch(BranchCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new graph
    Create(CreateCommand),
    /// Introspect a graph and print its schema
    Introspect(IntrospectCommand),
    /// List subgraphs
    Subgraphs(SubgraphsCommand),
    /// Fetch a federated graph or a subgraph
    Schema(SchemaCommand),
    /// Publish a subgraph schema
    Publish(PublishCommand),
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

impl ArgumentNames for SubCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        match self {
            SubCommand::Create(command) => command.argument_names(),
            SubCommand::Schema(_)
            | SubCommand::Branch(_)
            | SubCommand::Publish(_)
            | SubCommand::Check(_)
            | SubCommand::Subgraphs(_)
            | SubCommand::Introspect(_)
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Completions(_)
            | SubCommand::Trust(_)
            | SubCommand::Upgrade
            | SubCommand::Lint(_) => None,
        }
    }
}

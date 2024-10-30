use clap::Parser;

use crate::is_not_direct_install;

use super::{
    branch::BranchCommand, trust::TrustCommand, CheckCommand, CompletionsCommand, CreateCommand, IntrospectCommand,
    LintCommand, LoginCommand, PublishCommand, SchemaCommand, SubgraphsCommand,
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
    Login(LoginCommand),
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
                | SubCommand::Subgraphs(_)
                | SubCommand::Check(_)
                | SubCommand::Branch(_)
                | SubCommand::Schema(_)
        )
    }
}

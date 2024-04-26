use clap::{Parser, ValueEnum};

use super::GraphRefNoBranch;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, strum::AsRefStr, strum::Display)]
pub enum Environment {
    #[strum(serialize = "all")]
    All,
    #[strum(serialize = "production")]
    Production,
    #[strum(serialize = "preview")]
    Preview,
}

impl IntoIterator for Environment {
    type Item = &'static str;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Environment::All => vec![Environment::Production.as_ref(), Environment::Preview.as_ref()].into_iter(),
            Environment::Production => vec![Environment::Production.as_ref()].into_iter(),
            Environment::Preview => vec![Environment::Preview.as_ref()].into_iter(),
        }
    }
}

#[derive(Debug, Parser)]
pub struct EnvironmentCommand {
    #[command(subcommand)]
    pub command: EnvironmentSubCommand,
}

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
pub enum EnvironmentSubCommand {
    /// List all variables
    #[clap(visible_alias = "ls")]
    List(EnvironmentVariableListCommand),
    /// Create or update a variable
    Create(EnvironmentVariableCreateCommand),
    /// Remove a variable in the given branch environment
    #[clap(name = "remove", visible_alias = "rm")]
    Delete(EnvironmentVariableDeleteCommand),
}

#[derive(Debug, Parser)]
pub struct EnvironmentVariableCreateCommand {
    /// Specifies the graph. Defaults to linked graph, if not specified.
    #[arg(long, short, help = GraphRefNoBranch::ARG_DESCRIPTION)]
    pub graph_ref: Option<GraphRefNoBranch>,
    /// The environment where the variable is available
    #[clap(long, short, env = "GRAFBASE_ENVIRONMENT")]
    pub environment: Environment,
    /// The name of the variable
    pub name: String,
    /// The value of the variable
    pub value: String,
}

#[derive(Debug, Parser)]
pub struct EnvironmentVariableListCommand {
    /// Specifies the graph. Defaults to linked graph, if not specified.
    #[arg(long, short, help = GraphRefNoBranch::ARG_DESCRIPTION)]
    pub graph_ref: Option<GraphRefNoBranch>,
}

#[derive(Debug, Parser)]
pub struct EnvironmentVariableDeleteCommand {
    /// Specifies the graph. Defaults to linked graph, if not specified.
    #[arg(long, short, help = GraphRefNoBranch::ARG_DESCRIPTION)]
    pub graph_ref: Option<GraphRefNoBranch>,
    /// The name of the variable
    pub name: String,
    /// The environment where the variable is available
    #[clap(long, short, env = "GRAFBASE_ENVIRONMENT")]
    pub environment: Environment,
}

use clap::Parser;

use super::BranchRef;

#[derive(Debug, Parser)]
pub struct BranchCommand {
    #[command(subcommand)]
    pub command: BranchSubCommand,
}

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum BranchSubCommand {
    /// List all branches
    #[clap(name = "list", visible_alias = "ls")]
    List,
    /// Delete a branch
    #[clap(name = "remove", visible_alias = "rm")]
    Delete(BranchDeleteCommand),
}

#[derive(Debug, Parser)]
pub struct BranchDeleteCommand {
    /// Name of the branch
    #[arg(help = BranchRef::ARG_DESCRIPTION)]
    pub branch_ref: BranchRef,
}

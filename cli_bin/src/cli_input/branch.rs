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
    /// Create a branch on the linked graph (self-hosted graphs only)
    Create(BranchCreateCommand),
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

#[derive(Debug, Parser)]
pub struct BranchCreateCommand {
    /// Branch ref.
    #[arg(help = BranchRef::ARG_DESCRIPTION)]
    pub branch_ref: BranchRef,
}

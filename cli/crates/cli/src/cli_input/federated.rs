use clap::Parser;

pub use start::FederatedStartCommand;

mod start;

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum FederatedSubCommand {
    /// Start Grafbase in self-hosted mode
    Start(FederatedStartCommand),
}

#[derive(Debug, Parser)]
pub struct FederatedCommand {
    #[command(subcommand)]
    pub command: FederatedSubCommand,
}

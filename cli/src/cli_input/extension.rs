use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct ExtensionCommand {
    #[command(subcommand)]
    pub command: ExtensionSubCommand,
}

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ExtensionSubCommand {
    /// Create a new extension
    Init(ExtensionInitCommand),
}

#[derive(Debug, Parser)]
pub struct ExtensionInitCommand {
    /// The path where to create the extension
    pub path: PathBuf,
}

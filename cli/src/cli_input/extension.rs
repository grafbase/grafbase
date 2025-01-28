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
    /// Build the extension with options
    Build(ExtensionBuildCommand),
}

#[derive(Debug, Parser)]
pub struct ExtensionInitCommand {
    /// The path where to create the extension
    pub path: PathBuf,
}

#[derive(Debug, Parser)]
pub struct ExtensionBuildCommand {
    /// Output path for the built extension.
    #[arg(long, short, default_value = "./build")]
    pub output: PathBuf,
    /// Builds the extension in release mode.
    #[arg(long)]
    pub release: bool,
    /// Path to the extension source code.
    #[arg(long, short, default_value = ".")]
    pub path: PathBuf,
}

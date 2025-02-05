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
    /// The type of the extension
    #[arg(long, value_enum)]
    pub r#type: ExtensionType,
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ExtensionType {
    /// An extension that provides a field resolver
    Resolver,
    /// An extension that provides an authentication provider
    Auth,
}

#[derive(Debug, Parser)]
pub struct ExtensionBuildCommand {
    /// Output path for the built extension.
    #[arg(short, long, default_value = "./build")]
    pub output_dir: PathBuf,
    /// Builds the extension in debug mode.
    #[arg(long)]
    pub debug: bool,
    /// Path to the extension source code.
    #[arg(short, long, default_value = ".")]
    pub source_dir: PathBuf,
    /// Path to the extension scratch build directory.
    #[arg(long, default_value = "./target")]
    pub scratch_dir: PathBuf,
}

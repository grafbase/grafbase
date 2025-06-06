use std::path::PathBuf;

use clap::Parser;
use gateway_config::Config;

const DEFAULT_OUTPUT_DIR: &str = "./build";

#[derive(Debug, Parser)]
pub(crate) struct ExtensionCommand {
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
    /// Publish an extension
    Publish(ExtensionPublishCommand),
    /// Update the lockfile (grafbase-extensions.locks)
    Update(ExtensionUpdateCommand),
    /// Download the extensions captured in the lockfile.
    Install(ExtensionInstallCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct ExtensionInitCommand {
    /// The path where to create the extension
    pub path: PathBuf,
    /// The type of the extension
    #[arg(long, value_enum)]
    pub r#type: ExtensionType,
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum ExtensionType {
    /// An extension that provides a field resolver
    Resolver,
    /// An extension that provides an authentication provider
    Authentication,
    /// An extension for authorization
    Authorization,
    /// An extension for hooks
    Hooks,
}

#[derive(Debug, Parser)]
pub(crate) struct ExtensionBuildCommand {
    /// Output path for the built extension.
    #[arg(short, long, default_value = DEFAULT_OUTPUT_DIR)]
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

#[derive(Debug, Parser)]
pub(crate) struct ExtensionPublishCommand {
    /// Local path of the extension to publish. Typically the output dir of `grafbase extension build`.
    #[arg(default_value = DEFAULT_OUTPUT_DIR)]
    pub path: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ExtensionUpdateCommand {
    /// The name of the extension(s) to update. This argument can be passed multiple times. If no --name is passed, all extensions are updated.
    #[arg(short, long)]
    pub name: Option<Vec<String>>,
    /// The location of the gateway configuration file that contains the version requirements. Default: `./grafbase.toml` if it exists.
    #[arg(short('c'), long("config"))]
    config_path: Option<PathBuf>,
}

impl ExtensionUpdateCommand {
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::loader()
            .load_or_default(self.config_path.as_ref())
            .map_err(|err| anyhow::anyhow!(err))
    }
}

#[derive(Debug, Parser)]
pub(crate) struct ExtensionInstallCommand {
    /// The location of the gateway configuration file that contains the version requirements. Default: `./grafbase.toml` if it exists.
    #[arg(short('c'), long("config"))]
    config_path: Option<PathBuf>,
}

impl ExtensionInstallCommand {
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::loader()
            .load_or_default(self.config_path.as_ref())
            .map_err(|err| anyhow::anyhow!(err))
    }
}

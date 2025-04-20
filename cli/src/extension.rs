use crate::cli_input::{ExtensionCommand, ExtensionSubCommand};

mod build;
mod init;
pub mod install;
mod publish;
mod update;

const EXTENSION_WASM_MODULE_FILE_NAME: &str = "extension.wasm";

#[tokio::main]
pub(crate) async fn execute(cmd: ExtensionCommand) -> anyhow::Result<()> {
    match cmd.command {
        ExtensionSubCommand::Init(cmd) => init::execute(cmd),
        ExtensionSubCommand::Build(cmd) => build::execute(cmd),
        ExtensionSubCommand::Publish(cmd) => publish::execute(cmd).await,
        ExtensionSubCommand::Update(cmd) => update::execute(cmd).await,
        ExtensionSubCommand::Install(cmd) => install::execute(&cmd.config()?).await,
    }
}

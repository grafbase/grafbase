use crate::cli_input::{ExtensionCommand, ExtensionSubCommand};

mod build;
mod init;
mod publish;

const EXTENSION_WASM_MODULE_FILE_NAME: &str = "extension.wasm";

pub(crate) fn execute(cmd: ExtensionCommand) -> anyhow::Result<()> {
    match cmd.command {
        ExtensionSubCommand::Init(cmd) => init::execute(cmd),
        ExtensionSubCommand::Build(cmd) => build::execute(cmd),
        ExtensionSubCommand::Publish(cmd) => publish::execute(cmd),
    }
}

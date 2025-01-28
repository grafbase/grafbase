use crate::cli_input::{ExtensionCommand, ExtensionSubCommand};

mod build;
mod init;

pub(crate) fn execute(cmd: ExtensionCommand) -> anyhow::Result<()> {
    match cmd.command {
        ExtensionSubCommand::Init(cmd) => init::execute(cmd),
        ExtensionSubCommand::Build(cmd) => build::execute(cmd),
    }
}

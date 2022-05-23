use crate::cli_input::build_cli;
use crate::errors::CliError;
use clap_generate::generators::{generate, Bash, Elvish, Fish, PowerShell, Zsh};
use clap_generate::Generator;
use std::io;

/// generates shell specific completions for the cli and prints them to stdout
pub fn generate_completions(shell: &str) -> Result<(), CliError> {
    match shell {
        "bash" => completions_for_shell(Bash),
        "fish" => completions_for_shell(Fish),
        "zsh" => completions_for_shell(Zsh),
        "elvish" => completions_for_shell(Elvish),
        "powershell" => completions_for_shell(PowerShell),
        _ => return Err(CliError::UnsupportedShellForCompletions(shell.to_owned())),
    };
    Ok(())
}

fn completions_for_shell(generator: impl Generator) {
    generate(generator, &mut build_cli(), "grafbase", &mut io::stdout());
}

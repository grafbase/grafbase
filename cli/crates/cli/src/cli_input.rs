use cfg_if::cfg_if;
use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use indoc::indoc;

/// creates the cli interface
#[must_use]
pub fn build_cli() -> Command {
    cfg_if! {
        if #[cfg(debug_assertions)] {
            let command_builder = command!().arg(arg!(-t --trace "Activate tracing").action(ArgAction::SetTrue));
        } else {
            let command_builder = command!();
        }
    }

    command_builder
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("dev").about("Run your grafbase project locally").args(&[
                arg!(-p --port <port> "Use a specific port")
                    .default_value("4000")
                    .value_parser(value_parser!(u16)),
                arg!(-s --search "If a given port is unavailable, search for another").action(ArgAction::SetTrue),
                Arg::new("disable-watch")
                    .long("disable-watch")
                    .action(ArgAction::SetTrue)
                    .help("Do not listen for schema changes and reload"),
            ]),
        )
        .subcommand(
            Command::new("completions")
                .arg(Arg::new("shell").help(indoc! {"
                        The shell to generate completions for. 
                        Supported: bash, fish, zsh, elvish, powershell
                    "}))
                .arg_required_else_help(true)
                .about(indoc! {"
                    Output completions for the chosen shell
                    To use, write the output to the appropriate location for your shell
                "}),
        )
        .subcommand(
            Command::new("init")
                .args(&[
                    arg!([name] "The name of the project to create"),
                    arg!(-t --template <name> "The name or GitHub URL of the template to use for the new project"),
                ])
                .about(indoc! {"
                    Sets up the current or a new project for Grafbase
                "}),
        )
        .subcommand(Command::new("reset").about(indoc! {"
            Resets the local data for the current project by removing the .grafbase directory
        "}))
    // .subcommand(Command::new("login").about("TBD"))
    // .subcommand(Command::new("deploy").about("TBD"))
    // .subcommand(Command::new("logs").about("TBD"))
    // // TODO: schema edit / view
}

#[test]
fn verify_cli() {
    build_cli().debug_assert();
}

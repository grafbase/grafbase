use cfg_if::cfg_if;
use clap::{arg, command, value_parser, Arg, Command};
use indoc::indoc;

/// creates the cli interface
#[must_use]
pub fn build_cli() -> Command<'static> {
    let mut command_builder = command!();

    cfg_if! {
        if #[cfg(debug_assertions)]{
            command_builder = command_builder.arg(arg!(-t --trace "Activate tracing"))
        }
    }

    command_builder
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("dev").about("Run your grafbase project locally").args(&[
                arg!(-p --port <port> "Use a specific port")
                    .takes_value(true)
                    .default_value("4000")
                    .value_parser(value_parser!(u16))
                    .required(false),
                arg!(-s --search "If a given port is unavailable, search for another"),
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
                    arg!([name] "the name of the project to create"),
                    // arg!(-t --template "the name of the template to use for the new project"),
                ])
                .about(indoc! {"
                Sets up the current or a new project for Grafbase
            "}),
        )
    // .subcommand(Command::new("login").about("TBD"))
    // .subcommand(Command::new("deploy").about("TBD"))
    // .subcommand(Command::new("logs").about("TBD"))
    // // TODO: schema edit / view
}

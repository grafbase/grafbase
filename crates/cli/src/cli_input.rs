use clap::{arg, command, Arg, Command};
use indoc::indoc;

/// creates the cli interface
#[must_use]
pub fn build_cli() -> Command<'static> {
    command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("dev").about("Run your grafbase project locally").args(&[
            arg!(-p --port <port> "Use a specific port").required(false),
            arg!(-s --search "If a given port is unavailable, search for another"),
        ]))
        .subcommand(
            Command::new("completions")
                .arg(Arg::new("shell").help(indoc! {"
                        The shell to generate completions for. 
                        Supported: bash, fish, zsh, elvish, powershell
                    "}))
                .about(indoc! {"
                    Output completions for the chosen shell
                    To use, write the output to the appropriate location for your shell
                "}),
        )
    // .subcommand(Command::new("login").about("TBD"))
    // .subcommand(
    //     Command::new("create")
    //         .args(&[
    //             arg!(<name> "the name of the project to create"),
    //             arg!(-t --template "the name of the template to use for the new project"),
    //         ])
    //         .about("TBD"),
    // )
    // .subcommand(Command::new("deploy").about("TBD"))
    // .subcommand(Command::new("logs").about("TBD"))
}

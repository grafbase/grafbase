#![forbid(unsafe_code)]

mod cli_input;
mod completions;
mod dev;
mod errors;
mod output;

#[macro_use]
extern crate log;

use cli_input::build_cli;
use colored::control::ShouldColorize;
use common::{environment::Environment, traits::ToExitCode};
use dev::dev;
use errors::CliError;
use output::report;
use std::process;

fn main() {
    ShouldColorize::from_env();

    let exit_code = match try_main() {
        Ok(_) => exitcode::OK,
        Err(error) => {
            report::error(&error);
            error.to_exit_code()
        }
    };

    process::exit(exit_code);
}

fn try_main() -> Result<(), CliError> {
    pretty_env_logger::init();

    let matches = build_cli().get_matches();

    Environment::try_init().map_err(CliError::CommonError)?;

    match matches.subcommand() {
        Some(("dev", matches)) => {
            let search = matches.is_present("search");
            let port = matches
                .value_of("port")
                .map(str::parse::<u16>)
                .transpose()
                .map_err(|_| CliError::ParsePort)?;
            dev(search, port)
        }
        Some(("completions", matches)) => {
            let shell = matches.value_of("shell").unwrap();
            completions::generate(shell)
        }
        _ => unreachable!(),
    }
}

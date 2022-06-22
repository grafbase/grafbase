#![forbid(unsafe_code)]

mod cli_input;
mod completions;
mod dev;
mod errors;
mod output;

#[macro_use]
extern crate log;

use cli_input::build_cli;
use colorize::ShouldColorize;
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    environment::Environment,
    traits::ToExitCode,
};
use dev::dev;
use errors::CliError;
use output::report;
use std::process;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    panic_hook::setup!();

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
    let matches = build_cli().get_matches();

    let filter = EnvFilter::builder().parse_lossy(if matches.contains_id("trace") {
        TRACE_LOG_FILTER
    } else {
        DEFAULT_LOG_FILTER
    });

    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();

    report::cli_header();

    // running completions before initializing the environment
    // to prevent errors outside of a grafbase project
    if let Some(("completions", matches)) = matches.subcommand() {
        let shell = matches.get_one::<String>("shell").expect("must be present");
        completions::generate(shell)?;
        return Ok(());
    };

    Environment::try_init().map_err(CliError::CommonError)?;

    match matches.subcommand() {
        Some(("dev", matches)) => {
            // ignoring any errors to fall back to the normal handler if there's an issue
            let _set_handler_result = ctrlc::set_handler(|| {
                report::goodbye();
                process::exit(exitcode::OK);
            });

            let search = matches.contains_id("search");
            let port = matches.get_one::<u16>("port").copied();
            dev(search, port)
        }
        _ => unreachable!(),
    }
}

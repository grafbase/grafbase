#![forbid(unsafe_code)]

mod cli_input;
mod completions;
mod dev;
mod errors;
mod init;
mod output;
mod panic_hook;
mod watercolor;

#[macro_use]
extern crate log;

use cli_input::build_cli;
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    environment::Environment,
    traits::ToExitCode,
};
use dev::dev;
use errors::CliError;
use init::init;
use output::report;
use std::{convert::AsRef, process};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use watercolor::ShouldColorize;

fn main() {
    panic_hook!();

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

    if let Some(("completions", matches)) = matches.subcommand() {
        let shell = matches.get_one::<String>("shell").expect("must be present");
        completions::generate(shell)?;
        return Ok(());
    }

    report::cli_header();

    // commands that do not need an initialized environment
    match matches.subcommand() {
        Some(("init", matches)) => {
            let name = matches.get_one::<String>("name").map(AsRef::as_ref);
            return init(name);
        }
        _ => {}
    }

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

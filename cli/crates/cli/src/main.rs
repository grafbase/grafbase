#![forbid(unsafe_code)]

mod cli_input;
mod completions;
mod dev;
mod errors;
mod init;
mod login;
mod output;
mod panic_hook;
mod reset;
mod watercolor;

#[macro_use]
extern crate log;

use crate::reset::reset;
use cli_input::build_cli;
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    traits::ToExitCode,
};
use dev::dev;
use errors::CliError;
use init::init;
use login::login;
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

    let filter = EnvFilter::builder().parse_lossy(if let Ok(Some(true)) = matches.try_get_one("trace") {
        TRACE_LOG_FILTER
    } else {
        DEFAULT_LOG_FILTER
    });

    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();

    let subcommand = matches.subcommand();

    trace!("subcommand: {}", subcommand.expect("required").0);

    if let Some(("dev" | "init" | "reset" | "login"..)) = subcommand {
        report::cli_header();
    }

    match subcommand {
        Some(("completions", matches)) => {
            let shell = matches.get_one::<String>("shell").expect("must be present");

            completions::generate(shell)
        }
        Some(("dev", matches)) => {
            // ignoring any errors to fall back to the normal handler if there's an issue
            let _set_handler_result = ctrlc::set_handler(|| {
                report::goodbye();
                process::exit(exitcode::OK);
            });

            let search = matches.get_flag("search");
            let watch = !matches.get_flag("disable-watch");
            let port = matches.get_one::<u16>("port").copied();

            dev(search, watch, port)
        }
        Some(("init", matches)) => {
            let name = matches.get_one::<String>("name").map(AsRef::as_ref);
            let template = matches.get_one::<String>("template").map(AsRef::as_ref);
            init(name, template)
        }
        Some(("reset", _)) => reset(),
        Some(("login", _)) => login(),
        _ => unreachable!(),
    }
}

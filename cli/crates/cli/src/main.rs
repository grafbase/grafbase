#![forbid(unsafe_code)]

mod cli_input;
mod completions;
mod create;
mod deploy;
mod dev;
mod errors;
mod init;
mod link;
mod login;
mod logout;
mod output;
mod panic_hook;
mod prompts;
mod reset;
mod unlink;
mod watercolor;

#[macro_use]
extern crate log;

use crate::{
    create::{create, CreateArguments},
    deploy::deploy,
    dev::dev,
    init::init,
    link::link,
    login::login,
    logout::logout,
    reset::reset,
    unlink::unlink,
};
use cli_input::build_cli;
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    environment::Environment,
};
use errors::CliError;
use output::report;
use std::{convert::AsRef, process};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use watercolor::ShouldColorize;

fn main() {
    panic_hook!();

    ShouldColorize::from_env();

    let exit_code = match try_main() {
        Ok(_) => 0,
        Err(error) => {
            report::error(&error);
            1
        }
    };

    process::exit(exit_code);
}

fn try_main() -> Result<(), CliError> {
    let matches = build_cli().get_matches();
    let trace = matches.try_get_one::<u16>("trace");
    let filter = EnvFilter::builder().parse_lossy(if matches!(trace, Ok(Some(level)) if *level >= 1) {
        TRACE_LOG_FILTER
    } else {
        DEFAULT_LOG_FILTER
    });

    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();

    let subcommand = matches.subcommand();

    trace!("subcommand: {}", subcommand.expect("required").0);

    if let Some(("dev" | "init" | "reset" | "login" | "logout" | "create" | "deploy" | "link" | "unlink", ..)) =
        subcommand
    {
        report::cli_header();
    }

    if let Some(("dev" | "create" | "deploy" | "link" | "unlink", ..)) = subcommand {
        Environment::try_init().map_err(CliError::CommonError)?;
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

            dev(search, watch, port, matches!(trace, Ok(Some(level)) if *level >= 2))
        }
        Some(("init", matches)) => {
            let name = matches.get_one::<String>("name").map(AsRef::as_ref);
            let template = matches.get_one::<String>("template").map(AsRef::as_ref);
            init(name, template)
        }
        Some(("reset", _)) => reset(),
        Some(("login", _)) => login(),
        Some(("logout", _)) => logout(),
        Some(("create", matches)) => {
            let arguments = matches
                .get_one::<String>("account")
                .map(AsRef::as_ref)
                .zip(matches.get_one::<String>("name").map(AsRef::as_ref))
                // TODO change this once we support multiple regions from the CLI
                .zip(matches.get_one::<String>("regions").map(AsRef::as_ref))
                .map(|((account_slug, name), regions)| CreateArguments {
                    account_slug,
                    name,
                    // TODO change this once we support multiple regions from the CLI
                    regions: vec![regions],
                });
            create(&arguments)
        }
        Some(("deploy", _)) => deploy(),
        Some(("link", _)) => link(),
        Some(("unlink", _)) => unlink(),
        _ => unreachable!(),
    }
}

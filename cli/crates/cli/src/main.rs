#![cfg_attr(test, allow(unused_crate_dependencies))]
#![forbid(unsafe_code)]

use grafbase_workspace_hack as _;

mod branch;
mod check;
mod cli_input;
mod create;
mod dev;
mod errors;
mod introspect;
mod lint;
mod login;
mod logout;
mod output;
mod panic_hook;
mod prompts;
mod publish;
mod schema;
mod subgraphs;
mod trust;
mod upgrade;
mod watercolor;

#[macro_use]
extern crate log;

use crate::{
    cli_input::{Args, BranchSubCommand, RequiresLogin, SubCommand},
    create::create,
    login::login,
    logout::logout,
};
use clap::Parser;
use common::environment::{Environment, PlatformData};
use errors::CliError;
use output::report;
use std::{io::IsTerminal as _, path::PathBuf, process};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use watercolor::ShouldColorize;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    panic_hook!();

    rustls::crypto::ring::default_provider().install_default().unwrap();

    let args = Args::parse();
    ShouldColorize::from_env();

    let exit_code = match try_main(args) {
        Ok(()) => 0,
        Err(error) => {
            report::error(&error);
            1
        }
    };

    process::exit(exit_code);
}

fn try_main(args: Args) -> Result<(), CliError> {
    let filter = {
        let builder = EnvFilter::builder();
        match args.log_filter() {
            Some(argument_filter) => builder.parse_lossy(argument_filter),
            None => builder.from_env_lossy(),
        }
    };

    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();

    trace!("subcommand: {}", args.command);

    // do not display header if we're in a pipe
    if std::io::stdout().is_terminal() {
        report::cli_header();
    }

    Environment::try_init(args.home).map_err(CliError::CommonError)?;

    if args.command.requires_login() {
        PlatformData::try_init().map_err(CliError::CommonError)?;
    } else if matches!(args.command, SubCommand::Login) {
        PlatformData::try_init_ignore_credentials().map_err(CliError::CommonError)?;
    }

    report::warnings(&Environment::get().warnings);

    match args.command {
        SubCommand::Completions(cmd) => {
            cmd.shell.completions();

            Ok(())
        }
        SubCommand::Login => login(),
        SubCommand::Logout => logout(),
        SubCommand::Create(cmd) => create(&cmd.create_arguments()),
        SubCommand::Subgraphs(cmd) => subgraphs::subgraphs(cmd),
        SubCommand::Schema(cmd) => schema::schema(cmd),
        SubCommand::Publish(cmd) => publish::publish(cmd),
        SubCommand::Introspect(cmd) => introspect::introspect(&cmd),
        SubCommand::Check(cmd) => check::check(cmd),
        SubCommand::Trust(cmd) => trust::trust(cmd),
        SubCommand::Upgrade => {
            // this command is also hidden in this case
            // (clippy doesn't have a mechanism to completely disable a command conditionally when using derive, see https://github.com/clap-rs/clap/issues/5251)
            if is_not_direct_install() {
                return Err(CliError::NotDirectInstall);
            }
            upgrade::install_grafbase().map_err(Into::into)
        }
        SubCommand::Lint(cmd) => lint::lint(cmd.schema),
        SubCommand::Branch(cmd) => match cmd.command {
            BranchSubCommand::Delete(cmd) => branch::delete(cmd.branch_ref),
            BranchSubCommand::Create(cmd) => branch::create(cmd.branch_ref),
        },
        SubCommand::Dev(cmd) => dev::dev(cmd),
    }
}

pub(crate) fn is_not_direct_install() -> bool {
    std::env::current_exe().is_ok_and(|path| Some(path) != direct_install_executable_path())
}

pub(crate) fn direct_install_executable_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".grafbase").join("bin").join("grafbase"))
}

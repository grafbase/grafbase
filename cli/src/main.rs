#![forbid(unsafe_code)]

mod api;
mod branch;
mod check;
mod cli_input;
mod common;
mod compose;
mod create;
mod dev;
mod errors;
mod extension;
mod introspect;
mod lint;
mod login;
mod logout;
mod mcp;
mod output;
mod panic_hook;
mod plugins;
mod prompts;
mod publish;
mod schema;
mod subgraph;
mod trust;
mod upgrade;
mod watercolor;

#[macro_use]
extern crate log;

use crate::{
    cli_input::{Args, BranchSubCommand, RequiresLogin, SubCommand, SubgraphSubCommand},
    create::create,
    login::login,
    logout::logout,
};
use clap::Parser;
use common::{
    consts::OUTPUT_LAYER_LOG_FILTER,
    environment::{Environment, PlatformData},
    log::LogStyle,
};
use errors::CliError;
use output::report;
use std::{io::IsTerminal as _, path::PathBuf, process};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use watercolor::ShouldColorize;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    panic_hook!();

    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

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

    // logs meant to always reach output, e.g. user facing updates from background tasks
    let output_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_ansi(true)
        .with_filter(EnvFilter::new(OUTPUT_LAYER_LOG_FILTER));

    let logging_filter = filter.to_string();

    // Determine log style based on args or environment
    let log_style = args.log_style.unwrap_or_else(|| {
        let is_terminal = std::io::stdout().is_terminal();
        if is_terminal && (logging_filter.contains("debug") || logging_filter.contains("trace")) {
            LogStyle::Pretty
        } else {
            LogStyle::Text
        }
    });

    let registry = tracing_subscriber::registry().with(output_layer);

    match log_style {
        LogStyle::Pretty => registry
            .with(
                fmt::layer()
                    .pretty()
                    .with_ansi(std::io::stdout().is_terminal())
                    .with_target(false)
                    .with_filter(filter),
            )
            .init(),
        LogStyle::Text => registry
            .with(
                fmt::layer()
                    .with_ansi(std::io::stdout().is_terminal())
                    .with_target(false)
                    .with_filter(filter),
            )
            .init(),
        LogStyle::Json => registry.with(fmt::layer().json().with_filter(filter)).init(),
    };

    let command = args.command;

    trace!("subcommand: {command}");

    // do not display header if we're in a pipe
    if std::io::stdout().is_terminal() {
        report::cli_header();
    }

    Environment::try_init(args.home).map_err(CliError::CommonError)?;

    if command.requires_login() {
        PlatformData::try_init().map_err(CliError::CommonError)?;
    }

    report::warnings(&Environment::get().warnings);

    match command {
        SubCommand::Completions(cmd) => {
            cmd.shell.completions();

            Ok(())
        }
        SubCommand::Login(cmd) => {
            PlatformData::try_init_ignore_credentials(cmd.url).map_err(CliError::CommonError)?;
            login()
        }
        SubCommand::Logout => logout(),
        SubCommand::Create(cmd) => create(&cmd.create_arguments()),
        SubCommand::Compose(cmd) => Ok(compose::compose(cmd)?),
        SubCommand::Subgraph(cmd) => match cmd.command {
            SubgraphSubCommand::List(cmd) => subgraph::list(cmd.graph_ref),
            SubgraphSubCommand::Delete(cmd) => subgraph::delete(cmd.graph_ref, cmd.name),
        },
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
        SubCommand::Plugins => Ok(plugins::list()?),
        SubCommand::Branch(cmd) => match cmd.command {
            BranchSubCommand::Delete(cmd) => branch::delete(cmd.branch_ref),
            BranchSubCommand::Create(cmd) => branch::create(cmd.branch_ref),
        },
        SubCommand::Dev(cmd) => dev::dev(cmd, logging_filter),
        SubCommand::Extension(cmd) => Ok(extension::execute(cmd)?),
        SubCommand::Mcp(cmd) => Ok(mcp::run(cmd)?),

        SubCommand::Plugin(args) => Ok(plugins::execute(&args)?),
    }
}

pub(crate) fn is_not_direct_install() -> bool {
    std::env::current_exe().is_ok_and(|path| Some(path) != direct_install_executable_path())
}

pub(crate) fn direct_install_executable_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".grafbase").join("bin").join("grafbase"))
}

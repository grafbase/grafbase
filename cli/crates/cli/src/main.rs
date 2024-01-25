#![cfg_attr(test, allow(unused_crate_dependencies))]
#![forbid(unsafe_code)]

mod build;
mod check;
mod cli_input;
mod create;
mod deploy;
mod dev;
mod dump_config;
mod errors;
mod init;
mod introspect;
mod link;
mod login;
mod logout;
mod logs;
mod output;
mod panic_hook;
mod prompts;
mod publish;
mod schema;
mod start;
mod subgraphs;
mod unlink;
mod watercolor;

#[macro_use]
extern crate log;

use crate::{
    build::build,
    cli_input::{Args, ArgumentNames, LogsCommand, SubCommand},
    create::create,
    deploy::deploy,
    dev::dev,
    init::init,
    link::link,
    login::login,
    logout::logout,
    logs::logs,
    start::start,
    unlink::unlink,
};
use clap::Parser;
use common::{analytics::Analytics, environment::Environment};
use errors::CliError;
use output::report;
use std::process;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use watercolor::ShouldColorize;

fn main() {
    panic_hook!();

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
    let filter = EnvFilter::builder().parse_lossy(args.log_filter());

    tracing_subscriber::registry().with(fmt::layer()).with(filter).init();
    trace!("subcommand: {}", args.command);

    // do not display header if we're in a pipe
    if atty::is(atty::Stream::Stdout) {
        report::cli_header();
    }

    if args.command.in_project_context() {
        Environment::try_init_with_project(args.home).map_err(CliError::CommonError)?;
    } else {
        Environment::try_init(args.home).map_err(CliError::CommonError)?;
    }

    Analytics::init().map_err(CliError::CommonError)?;
    Analytics::command_executed(args.command.as_ref(), args.command.argument_names());

    report::warnings(&Environment::get().warnings);

    match args.command {
        SubCommand::Completions(cmd) => {
            cmd.shell.completions();

            Ok(())
        }
        SubCommand::Dev(cmd) => {
            // ignoring any errors to fall back to the normal handler if there's an issue
            let _set_handler_result = ctrlc::set_handler(|| {
                report::goodbye();
                process::exit(exitcode::OK);
            });

            dev(
                cmd.search,
                !cmd.disable_watch,
                cmd.subgraph_port(),
                cmd.log_levels(),
                args.trace >= 2,
            )
        }
        SubCommand::Init(cmd) => init(cmd.name(), cmd.template(), cmd.graph),
        SubCommand::Login => login(),
        SubCommand::Logout => logout(),
        SubCommand::Create(cmd) => create(&cmd.create_arguments()),
        SubCommand::Deploy => deploy(),
        SubCommand::Link(cmd) => link(cmd.project),
        SubCommand::Unlink => unlink(),
        SubCommand::Logs(LogsCommand {
            project_branch,
            limit,
            no_follow,
        }) => logs(project_branch, limit, !no_follow),
        SubCommand::Start(cmd) => {
            let _ = ctrlc::set_handler(|| {
                report::goodbye();
                process::exit(exitcode::OK);
            });

            start(
                cmd.listen_address(),
                cmd.port,
                cmd.log_levels(),
                cmd.federated_graph_schema_path(),
                args.trace >= 2,
            )
        }
        SubCommand::Build(cmd) => {
            let _ = ctrlc::set_handler(|| {
                report::goodbye();
                process::exit(exitcode::OK);
            });

            build(cmd.parallelism(), args.trace >= 2)
        }
        SubCommand::Subgraphs(cmd) => subgraphs::subgraphs(cmd),
        SubCommand::Schema(cmd) => schema::schema(cmd),
        SubCommand::Publish(cmd) => {
            if cmd.dev {
                report::publishing();
                match federated_dev::add_subgraph(
                    &cmd.subgraph_name,
                    &cmd.url,
                    cmd.dev_api_port,
                    cmd.headers().collect(),
                ) {
                    Ok(_) => {
                        report::publish_command_success(&cmd.subgraph_name);
                        Ok(())
                    }
                    Err(federated_dev::Error::Internal(error)) => {
                        report::local_publish_command_failure(&cmd.subgraph_name, &error.to_string());
                        Ok(())
                    }
                    Err(federated_dev::Error::SubgraphComposition(error)) => {
                        report::local_publish_command_failure(&cmd.subgraph_name, &error.to_string());
                        Ok(())
                    }
                    Err(other) => Err(CliError::Publish(other.to_string())),
                }
            } else {
                publish::publish(cmd)
            }
        }
        SubCommand::Introspect(cmd) => introspect::introspect(&cmd),
        SubCommand::DumpConfig => dump_config::dump_config(),
        SubCommand::Check(cmd) => check::check(cmd),
    }
}

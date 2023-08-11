use crate::create::CreateArguments;
use clap::{arg, command, CommandFactory, Parser, ValueEnum};
use clap_complete::{shells, Generator};
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    types::LogLevel,
};
use std::{fmt, path::PathBuf};

const DEFAULT_PORT: u16 = 4000;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum LogLevelFilter {
    None,
    Error,
    Warn,
    Info,
    Debug,
}

impl From<LogLevelFilter> for Option<LogLevel> {
    fn from(value: LogLevelFilter) -> Self {
        match value {
            LogLevelFilter::None => None,
            LogLevelFilter::Error => Some(LogLevel::Error),
            LogLevelFilter::Warn => Some(LogLevel::Warn),
            LogLevelFilter::Info => Some(LogLevel::Info),
            LogLevelFilter::Debug => Some(LogLevel::Debug),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LogLevelFilters {
    pub functions: Option<LogLevel>,
    pub graphql_operations: Option<LogLevel>,
    pub fetch_requests: Option<LogLevel>,
}

#[derive(Debug, Parser)]
pub struct DevCommand {
    /// Use a specific port
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    /// If a given port is unavailable, search for another
    #[arg(short, long)]
    pub search: bool,
    /// Do not listen for schema changes and reload
    #[arg(long)]
    pub disable_watch: bool,
    /// Log level to print from function invocations, defaults to 'log-level'
    #[arg(long, value_name = "FUNCTION_LOG_LEVEL")]
    pub log_level_functions: Option<LogLevelFilter>,
    /// Log level to print for GraphQL operations, defaults to 'log-level'
    #[arg(long, value_name = "GRAPHQL_OPERATION_LOG_LEVEL")]
    pub log_level_graphql_operations: Option<LogLevelFilter>,
    /// Log level to print for fetch requests, defaults to 'log-level'
    #[arg(long, value_name = "GRAPHQL_OPERATION_LOG_LEVEL")]
    pub log_level_fetch_requests: Option<LogLevelFilter>,
    /// Default log level to print
    #[arg(long)]
    pub log_level: Option<LogLevelFilter>,
    /// A shortcut to enable fairly detailed logging
    #[arg(short, long)]
    pub verbose: bool,
}

impl DevCommand {
    pub fn log_levels(&self) -> LogLevelFilters {
        let default_log_level = if self.verbose {
            LogLevelFilter::Debug
        } else {
            LogLevelFilter::Info
        };
        LogLevelFilters {
            functions: self.log_level_functions.unwrap_or(default_log_level).into(),
            graphql_operations: self.log_level_graphql_operations.unwrap_or(default_log_level).into(),
            fetch_requests: self.log_level_fetch_requests.unwrap_or(default_log_level).into(),
        }
    }
}

#[derive(Debug, Parser, Clone, Copy)]
pub enum Shell {
    /// Generate completions for bash
    Bash,
    /// Generate completions for fish
    Fish,
    /// Generate completions for zsh
    Zsh,
    /// Generate completions for elvish
    Elvish,
    /// Generate completions for powershell
    #[command(name = "powershell")]
    PowerShell,
}

impl Shell {
    pub fn completions(self) {
        match self {
            Shell::Bash => Self::completions_for_shell(shells::Bash),
            Shell::Fish => Self::completions_for_shell(shells::Fish),
            Shell::Zsh => Self::completions_for_shell(shells::Zsh),
            Shell::Elvish => Self::completions_for_shell(shells::Elvish),
            Shell::PowerShell => Self::completions_for_shell(shells::PowerShell),
        }
    }

    fn completions_for_shell(generator: impl Generator) {
        clap_complete::generate(generator, &mut Args::command(), "grafbase", &mut std::io::stdout());
    }
}

#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// The shell to generate completions for
    #[command(subcommand)]
    pub shell: Shell,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum ConfigFormat {
    /// Adds a TypeScript configuration file
    TypeScript,
    /// Adds a GraphQL configuration file
    GraphQL,
}

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// The name of the project to create
    pub name: Option<String>,
    /// The name or GitHub URL of the template to use for the new project
    #[arg(short, long)]
    pub template: Option<String>,
    /// The format used for the Grafbase configuration file
    #[arg(short, long)]
    pub config_format: Option<ConfigFormat>,
}

impl InitCommand {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn template(&self) -> Option<&str> {
        self.template.as_deref()
    }
}

#[derive(Debug, clap::Args)]
#[group(requires_all = ["name", "account", "regions"], multiple = true)]
pub struct CreateCommand {
    /// The name to use for the new project
    #[arg(short, long)]
    pub name: Option<String>,
    /// The slug of the account in which the new project should be created
    #[arg(short, long, value_name = "SLUG")]
    pub account: Option<String>,
    /// The regions in which the database for the new project should be created
    #[arg(short, long, value_name = "REGION")]
    pub regions: Option<Vec<String>>,
}

impl CreateCommand {
    pub fn create_arguments(&self) -> Option<CreateArguments<'_>> {
        self.name
            .as_deref()
            .zip(self.account.as_deref())
            .zip(self.regions.as_deref())
            .map(|((name, account_slug), regions)| CreateArguments {
                account_slug,
                name,
                regions,
            })
    }
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Run your Grafbase project locally
    Dev(DevCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Sets up the current or a new project for Grafbase
    Init(InitCommand),
    /// Resets the local database for the current project
    Reset,
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new project
    Create(CreateCommand),
    /// Deploy your project
    Deploy,
    /// Connect a local project to a remote project
    Link,
    /// Disconnect a local project from a remote project
    Unlink,
}

// TODO see if there's a way to do this automatically (https://github.com/clap-rs/clap/discussions/4921)
pub trait ArgumentNames {
    /// returns the argument names used in a specific invocation of the CLI
    fn argument_names(&self) -> Option<Vec<&'static str>>;
}

fn filter_existing_arguments(arguments: &[(bool, &'static str)]) -> Option<Vec<&'static str>> {
    let arguments = arguments
        .iter()
        .filter(|arguments| arguments.0)
        .map(|arguments| arguments.1)
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        None
    } else {
        Some(arguments)
    }
}

impl ArgumentNames for DevCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[
            (self.port != DEFAULT_PORT, "port"),
            (self.search, "search"),
            (self.disable_watch, "disable-watch"),
        ])
    }
}

impl ArgumentNames for InitCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[(self.name.is_some(), "name"), (self.template.is_some(), "template")])
    }
}

impl ArgumentNames for CreateCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        let arguments = [(self.name.is_some(), vec!["name", "account", "regions"])]
            .iter()
            .filter(|arguments| arguments.0)
            .flat_map(|arguments| arguments.1.clone())
            .collect::<Vec<_>>();
        if arguments.is_empty() {
            None
        } else {
            Some(arguments)
        }
    }
}

impl ArgumentNames for SubCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        match self {
            SubCommand::Dev(command) => command.argument_names(),
            SubCommand::Init(command) => command.argument_names(),
            SubCommand::Create(command) => command.argument_names(),
            SubCommand::Reset
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Deploy
            | SubCommand::Link
            | SubCommand::Unlink
            | SubCommand::Completions(_) => None,
        }
    }
}

impl SubCommand {
    pub(crate) fn in_project_context(&self) -> bool {
        matches!(
            self,
            Self::Dev(_) | Self::Create(_) | Self::Deploy | Self::Link | Self::Unlink | Self::Reset
        )
    }
}

impl AsRef<str> for SubCommand {
    fn as_ref(&self) -> &str {
        match self {
            SubCommand::Dev(_) => "dev",
            SubCommand::Completions(_) => "completions",
            SubCommand::Init(_) => "init",
            SubCommand::Reset => "reset",
            SubCommand::Login => "login",
            SubCommand::Logout => "logout",
            SubCommand::Create(_) => "create",
            SubCommand::Deploy => "deploy",
            SubCommand::Link => "link",
            SubCommand::Unlink => "unlink",
        }
    }
}

impl fmt::Display for SubCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Parser)]
#[command(name = "Grafbase CLI", version)]
/// The Grafbase command line interface
pub struct Args {
    /// Set the tracing level
    #[arg(short, long, default_value_t = 0)]
    pub trace: u16,
    #[command(subcommand)]
    pub command: SubCommand,
    /// An optional replacement path for the home directory
    #[arg(long)]
    pub home: Option<PathBuf>,
}

impl Args {
    pub fn log_filter(&self) -> &str {
        if self.trace >= 1 {
            TRACE_LOG_FILTER
        } else {
            DEFAULT_LOG_FILTER
        }
    }
}

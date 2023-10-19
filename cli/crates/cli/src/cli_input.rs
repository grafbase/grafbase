use crate::create::CreateArguments;
use clap::{arg, command, CommandFactory, Parser, ValueEnum};
use clap_complete::{shells, Generator};
use common::{
    consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER},
    types::LogLevel,
};
use std::{
    net::{IpAddr, Ipv4Addr},
    num::NonZeroUsize,
    path::PathBuf,
};
use ulid::Ulid;

const DEFAULT_PORT: u16 = 4000;

#[derive(Default, Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum LogLevelFilter {
    None,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
}

impl LogLevelFilter {
    pub fn should_display(self, level: LogLevel) -> bool {
        Some(level)
            <= (match self {
                LogLevelFilter::None => None,
                LogLevelFilter::Error => Some(LogLevel::Error),
                LogLevelFilter::Warn => Some(LogLevel::Warn),
                LogLevelFilter::Info => Some(LogLevel::Info),
                LogLevelFilter::Debug => Some(LogLevel::Debug),
            })
    }
}

#[derive(Default, Clone, Copy)]
pub struct LogLevelFilters {
    pub functions: LogLevelFilter,
    pub graphql_operations: LogLevelFilter,
    pub fetch_requests: LogLevelFilter,
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
    #[arg(long, value_name = "FETCH_REQUEST_LOG_LEVEL")]
    pub log_level_fetch_requests: Option<LogLevelFilter>,
    /// Default log level to print
    #[arg(long)]
    pub log_level: Option<LogLevelFilter>,
    /// A shortcut to enable fairly detailed logging
    #[arg(short, long, conflicts_with = "log_level")]
    pub verbose: bool,
}

impl DevCommand {
    pub fn log_levels(&self) -> LogLevelFilters {
        let default_log_levels = if self.verbose {
            LogLevelFilters {
                functions: LogLevelFilter::Debug,
                graphql_operations: LogLevelFilter::Info,
                fetch_requests: LogLevelFilter::Debug,
            }
        } else {
            LogLevelFilters {
                functions: self.log_level.unwrap_or_default(),
                graphql_operations: self.log_level.unwrap_or_default(),
                fetch_requests: self.log_level.unwrap_or_default(),
            }
        };
        LogLevelFilters {
            functions: self.log_level_functions.unwrap_or(default_log_levels.functions),
            graphql_operations: self
                .log_level_graphql_operations
                .unwrap_or(default_log_levels.graphql_operations),
            fetch_requests: self
                .log_level_fetch_requests
                .unwrap_or(default_log_levels.fetch_requests),
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

#[derive(Debug, clap::Args)]
pub struct LinkCommand {
    /// The id of the linked project
    #[arg(short, long, value_name = "PROJECT_ID")]
    pub project: Option<Ulid>,
}

const DEFAULT_LOGS_LIMIT: u16 = 100;

#[derive(Debug, clap::Args)]
pub struct LogsCommand {
    /// The reference to a project: either `{account_slug}/{project_slug}`, `{project_slug}` for the personal account, or a URL to a deployed gateway.
    /// Defaults to the linked project if there's one.
    #[arg(value_name = "PROJECT_BRANCH")]
    pub project_branch: Option<String>,
    /// How many last entries to retrive
    #[arg(short, long, default_value_t = DEFAULT_LOGS_LIMIT)]
    pub limit: u16,
    /// Whether to disable polling for new log entries
    #[arg(long)]
    pub no_follow: bool,
}

#[derive(Debug, clap::Args)]
pub struct BuildCommand {
    /// Number of resolver builds running in parallel
    #[arg(long)]
    pub parallelism: Option<u16>,
}

impl BuildCommand {
    pub fn parallelism(&self) -> NonZeroUsize {
        let parallelism = self.parallelism.unwrap_or(0);
        if parallelism == 0 {
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).expect("strictly positive"))
        } else {
            NonZeroUsize::new(parallelism as usize).expect("strictly positive")
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct StartCommand {
    /// Use a specific port
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    /// Log level to print from function invocations, defaults to 'log-level'
    #[arg(long, value_name = "FUNCTION_LOG_LEVEL")]
    pub log_level_functions: Option<LogLevelFilter>,
    /// Log level to print for GraphQL operations, defaults to 'log-level'
    #[arg(long, value_name = "GRAPHQL_OPERATION_LOG_LEVEL")]
    pub log_level_graphql_operations: Option<LogLevelFilter>,
    /// Log level to print for fetch requests, defaults to 'log-level'
    #[arg(long, value_name = "FETCH_REQUEST_LOG_LEVEL")]
    pub log_level_fetch_requests: Option<LogLevelFilter>,
    /// Default log level to print
    #[arg(long)]
    pub log_level: Option<LogLevelFilter>,
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1.
    #[arg(long)]
    pub listen_address: Option<IpAddr>,
}

impl StartCommand {
    pub fn log_levels(&self) -> LogLevelFilters {
        let default_log_levels = LogLevelFilters {
            functions: self.log_level.unwrap_or_default(),
            graphql_operations: self.log_level.unwrap_or_default(),
            fetch_requests: self.log_level.unwrap_or_default(),
        };
        LogLevelFilters {
            functions: self.log_level_functions.unwrap_or(default_log_levels.functions),
            graphql_operations: self
                .log_level_graphql_operations
                .unwrap_or(default_log_levels.graphql_operations),
            fetch_requests: self
                .log_level_fetch_requests
                .unwrap_or(default_log_levels.fetch_requests),
        }
    }

    pub fn listen_address(&self) -> IpAddr {
        self.listen_address.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }
}

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
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
    Link(LinkCommand),
    /// Disconnect a local project from a remote project
    Unlink,
    /// Tails logs from a remote project
    Logs(LogsCommand),
    /// Run your Grafbase project locally in production mode
    Start(StartCommand),
    /// Build the Grafbase project in advance to avoid the resolver build step in the start
    /// command.
    Build(BuildCommand),
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
            | SubCommand::Link(_)
            | SubCommand::Unlink
            | SubCommand::Start(_)
            | SubCommand::Build(_)
            | SubCommand::Completions(_)
            | SubCommand::Logs(_) => None,
        }
    }
}

impl SubCommand {
    pub(crate) fn in_project_context(&self) -> bool {
        matches!(
            self,
            Self::Create(_)
                | Self::Deploy
                | Self::Dev(_)
                | Self::Link(_)
                | Self::Logs(_)
                | Self::Reset
                | Self::Start(_)
                | Self::Build(_)
                | Self::Unlink
        )
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

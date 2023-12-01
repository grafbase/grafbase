use super::{filter_existing_arguments, ArgumentNames, LogLevelFilter, LogLevelFilters, DEFAULT_SUBGRAPH_PORT};
use clap::{arg, Parser};

#[derive(Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
pub struct DevCommand {
    /// Use a specific port
    #[arg(short, long)]
    pub port: Option<u16>,
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
                graphql_operations: LogLevelFilter::Debug,
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

    pub fn subgraph_port(&self) -> u16 {
        self.port.unwrap_or(DEFAULT_SUBGRAPH_PORT)
    }
}

impl ArgumentNames for DevCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[
            (self.subgraph_port() != DEFAULT_SUBGRAPH_PORT, "port"),
            (self.search, "search"),
            (self.disable_watch, "disable-watch"),
        ])
    }
}

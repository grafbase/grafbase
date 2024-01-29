use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use super::{LogLevelFilter, LogLevelFilters, DEFAULT_SUBGRAPH_PORT};

#[derive(Debug, clap::Args)]
pub struct StartCommand {
    /// Use a specific port
    #[arg(short, long, default_value_t = DEFAULT_SUBGRAPH_PORT)]
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
    /// Path to federated graph SDL. If provided, the graph will be static and cannot be updated.
    #[arg(long)]
    pub federated_graph_schema: Option<PathBuf>,
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
        self.listen_address.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    }

    pub fn federated_graph_schema_path(&self) -> Option<PathBuf> {
        self.federated_graph_schema
            .as_ref()
            .zip(std::env::current_dir().ok())
            .map(|(path, cwd)| cwd.join(path))
            .or(self.federated_graph_schema.clone())
    }
}

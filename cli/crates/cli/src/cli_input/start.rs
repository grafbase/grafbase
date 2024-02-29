use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use super::{LogLevelFilter, LogLevelFilters};

#[derive(Debug, clap::Args)]
pub struct StartCommand {
    /// Default log level to print
    #[arg(long)]
    pub log_level: Option<LogLevelFilter>,
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:4000.
    #[arg(long)]
    pub listen_address: Option<SocketAddr>,
    /// Path to federated graph SDL. If provided, the graph will be static and cannot be updated.
    #[arg(long)]
    pub federated_schema: Option<PathBuf>,
}

impl StartCommand {
    pub fn log_levels(&self) -> LogLevelFilters {
        LogLevelFilters {
            functions: self.log_level.unwrap_or_default(),
            graphql_operations: self.log_level.unwrap_or_default(),
            fetch_requests: self.log_level.unwrap_or_default(),
        }
    }

    pub fn listen_address(&self) -> SocketAddr {
        self.listen_address
            .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 4000))
    }

    pub fn federated_schema_path(&self) -> Option<PathBuf> {
        self.federated_schema
            .as_ref()
            .zip(std::env::current_dir().ok())
            .map(|(path, cwd)| cwd.join(path))
            .or(self.federated_schema.clone())
    }
}

use std::ops::Range;

/// the default port on which the server will run
pub const DEFAULT_PORT: u16 = 4000;
/// the max port to use when searching for an available port
pub const MAX_PORT: u16 = u16::MAX;
/// localhost IP
pub const LOCALHOST: &str = "127.0.0.1";
/// the name of the directory indicating a grafbase project
pub const GRAFBASE_DIRECTORY_NAME: &str = "grafbase";
/// a file expected to be in the grafbase directory
pub const GRAFBASE_SCHEMA_FILE_NAME: &str = "schema.graphql";
/// a file expected to be in the grafbase directory
pub const GRAFBASE_TS_CONFIG_FILE_NAME: &str = "grafbase.config.ts";
/// a file expected to be in the grafbase directory
pub const GRAFBASE_ENV_FILE_NAME: &str = ".env";
/// the name for the db / cache directory per project and the global cache directory for the user
pub const DOT_GRAFBASE_DIRECTORY: &str = ".grafbase";
/// the registry.json file generated from schema.graphql
pub const REGISTRY_FILE: &str = "registry.json";
/// the /resolvers directory containing resolver implementations
pub const RESOLVERS_DIRECTORY_NAME: &str = "resolvers";
/// the tracing filter to be used when tracing is on
pub const TRACE_LOG_FILTER: &str = "grafbase=trace,grafbase_local_common=trace,grafbase_local_server=trace,grafbase_local_backend=trace,tower_http=debug";
/// the tracing filter to be used when tracing is off
pub const DEFAULT_LOG_FILTER: &str = "off";
/// the range suggested for ephemeral ports by IANA
pub const EPHEMERAL_PORT_RANGE: Range<u16> = 49152..65535;
/// the subdirectory within '$PROJECT/.grafbase' containing the database
pub const DATABASE_DIRECTORY: &str = "database";

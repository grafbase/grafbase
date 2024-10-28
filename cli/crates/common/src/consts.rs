/// the default port on which the server will run
pub const DEFAULT_PORT: u16 = 4000;
/// the max port to use when searching for an available port
pub const MAX_PORT: u16 = u16::MAX;
/// localhost IP
pub const LOCALHOST: &str = "127.0.0.1";
/// the name for the db / cache directory per project and the global cache directory for the user
pub const DOT_GRAFBASE_DIRECTORY_NAME: &str = ".grafbase";
/// the tracing filter to be used when tracing is on
pub const TRACE_LOG_FILTER: &str = "info,grafbase=trace,grafbase_local_common=trace,grafbase_local_backend=trace,postgres-connector-types=trace,engine_v2=debug";
/// an environment variable that sets the path of the home directory
pub const GRAFBASE_HOME: &str = "GRAFBASE_HOME";
/// the user agent for CLI HTTP calls
pub const USER_AGENT: &str = const_format::formatcp!("Grafbase-CLI-{}", env!("CARGO_PKG_VERSION"));
/// the name of the login credentials file
pub const CREDENTIALS_FILE: &str = "credentials.json";
/// the env var used to set the access token
pub const GRAFBASE_ACCESS_TOKEN_ENV_VAR: &str = "GRAFBASE_ACCESS_TOKEN";

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
pub const DOT_GRAFBASE_DIRECTORY_NAME: &str = ".grafbase";
/// the registry.json file generated from schema.graphql
pub const REGISTRY_FILE: &str = "registry.json";
/// the /resolvers directory containing resolver implementations
pub const RESOLVERS_DIRECTORY_NAME: &str = "resolvers";
/// the /auth directory containing custom authorizers
pub const AUTHORIZERS_DIRECTORY_NAME: &str = "auth";
/// the wrangler installation directory within ~/.grafbase
pub const WRANGLER_DIRECTORY_NAME: &str = "wrangler";
/// the tracing filter to be used when tracing is on
pub const TRACE_LOG_FILTER: &str = "grafbase=trace,grafbase_local_common=trace,grafbase_local_server=trace,grafbase_local_backend=trace,tower_http=debug,federated_dev=trace,postgres-connector-types=trace,engine_v2=debug";
/// the tracing filter to be used when tracing is off
pub const DEFAULT_LOG_FILTER: &str = "off";
/// the subdirectory within '$PROJECT/.grafbase' containing the database
pub const DATABASE_DIRECTORY: &str = "database";
/// an environment variable that sets the path of the home directory
pub const GRAFBASE_HOME: &str = "GRAFBASE_HOME";
/// the name of the Grafbase SDK npm package
pub const GRAFBASE_SDK_PACKAGE_NAME: &str = "@grafbase/sdk";
/// the version string of the Grafbase SDK npm package
pub const GRAFBASE_SDK_PACKAGE_VERSION: &str = env!("GRAFBASE_SDK_PACKAGE_VERSION");
/// the package.json file name
pub const PACKAGE_JSON_FILE_NAME: &str = "package.json";
/// the package.json dev dependencies key
pub const PACKAGE_JSON_DEV_DEPENDENCIES: &str = "devDependencies";

/// The directory we generate the schema.graphql file inside
pub const GENERATED_SCHEMAS_DIR: &str = "generated/schemas";

/// the default port on which the dev server will run
pub const DEFAULT_PORT: u16 = 4000;
/// the max port to use when searching for an available port
pub const MAX_PORT: u16 = 9000;
/// localhost IP
pub const LOCALHOST: &str = "127.0.0.1";
/// the name of the folder indicating a grafbase project
pub const GRAFBASE_FOLDER: &str = "grafbase";
/// a file expected to be in the grafbase folder
pub const GRAFBASE_SCHEMA: &str = "schema.graphql";
/// the name for the db / cache directory per project and the global cache directory for the user
pub const DOT_GRAFBASE_FOLDER: &str = ".grafbase";

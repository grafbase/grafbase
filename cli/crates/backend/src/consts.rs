use const_format::formatcp;

pub const DEFAULT_SCHEMA: &str = include_str!("../assets/default-schema.graphql");
pub const CREDENTIALS_FILE: &str = "credentials.json";
pub const PROJECT_METADATA_FILE: &str = "project.json";
pub const AUTH_URL: &str = "https://grafbase.dev/auth/cli";
// TODO remove this
#[allow(unused)]
pub const API_URL: &str = "https://api.grafbase.dev/graphql";
pub const USER_AGENT: &str = formatcp!("Grafbase-CLI-{}", env!("CARGO_PKG_VERSION"));

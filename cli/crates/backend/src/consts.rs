use const_format::formatcp;

pub const DEFAULT_SCHEMA: &str = include_str!("../assets/default-schema.graphql");
pub const DEFAULT_DOT_ENV: &str = include_str!("../assets/default.env");
pub const USER_AGENT: &str = formatcp!("Grafbase-CLI-{}", env!("CARGO_PKG_VERSION"));

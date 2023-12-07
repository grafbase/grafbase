use const_format::formatcp;

pub const DEFAULT_SCHEMA_FEDERATED: &str = include_str!("../assets/grafbase.federated.config.ts");
pub const DEFAULT_SCHEMA_STANDALONE: &str = include_str!("../assets/grafbase.standalone.config.ts");
pub const DEFAULT_DOT_ENV: &str = include_str!("../assets/default.env");
pub const USER_AGENT: &str = formatcp!("Grafbase-CLI-{}", env!("CARGO_PKG_VERSION"));

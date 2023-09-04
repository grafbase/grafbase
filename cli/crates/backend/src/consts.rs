use const_format::formatcp;

pub const DEFAULT_SCHEMA_SDL: &str = include_str!("../assets/default-schema.graphql");
pub const DEFAULT_SCHEMA_TS: &str = include_str!("../assets/grafbase.default.config.ts");
pub const DEFAULT_HELLO_RESOLVER: &str = include_str!("../assets/hello.ts");
pub const DEFAULT_GRAVATAR_RESOLVER: &str = include_str!("../assets/gravatar.ts");
pub const DEFAULT_DOT_ENV: &str = include_str!("../assets/default.env");
pub const USER_AGENT: &str = formatcp!("Grafbase-CLI-{}", env!("CARGO_PKG_VERSION"));

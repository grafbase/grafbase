use std::time::Duration;

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
pub struct EntityCachingConfig {
    pub enabled: Option<bool>,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_option_duration", default)]
    pub ttl: Option<Duration>,
}

impl From<EntityCachingConfig> for parser_sdl::federation::EntityCachingConfig {
    fn from(config: EntityCachingConfig) -> Self {
        match (config.enabled, config.ttl) {
            (Some(false), _) => parser_sdl::federation::EntityCachingConfig::Disabled,
            (Some(true), ttl) => parser_sdl::federation::EntityCachingConfig::Enabled { ttl },
            (_, Some(ttl)) => parser_sdl::federation::EntityCachingConfig::Enabled { ttl: Some(ttl) },
            _ => parser_sdl::federation::EntityCachingConfig::Disabled,
        }
    }
}

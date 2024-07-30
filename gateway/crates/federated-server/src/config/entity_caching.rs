use std::time::Duration;

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
pub struct EntityCachingConfig {
    pub enabled: bool,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub ttl: Option<Duration>,
}

impl From<EntityCachingConfig> for parser_sdl::federation::EntityCachingConfig {
    fn from(config: EntityCachingConfig) -> Self {
        if config.enabled {
            parser_sdl::federation::EntityCachingConfig::Enabled { ttl: config.ttl }
        } else {
            parser_sdl::federation::EntityCachingConfig::Disabled
        }
    }
}

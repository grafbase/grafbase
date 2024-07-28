use std::time::Duration;

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
pub struct EntityCachingConfig {
    pub enabled: Option<bool>,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_option_duration", default)]
    pub ttl: Option<Duration>,
}

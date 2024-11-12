use std::time::Duration;

const DEFAULT_ENTITY_CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, Copy)]
pub enum EntityCaching {
    #[default]
    Disabled,
    Enabled {
        ttl: Option<Duration>,
    },
}

impl EntityCaching {
    pub fn ttl(&self) -> Option<Duration> {
        match self {
            Self::Enabled { ttl } => Some(ttl.unwrap_or(DEFAULT_ENTITY_CACHE_TTL)),
            _ => None,
        }
    }
}

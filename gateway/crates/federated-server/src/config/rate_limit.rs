use duration_str::deserialize_duration;
use serde::de::Error;
use serde::Deserializer;
use std::time::Duration;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RateLimitConfig {
    pub limit: usize,
    #[serde(deserialize_with = "deserialize_duration_internal")]
    pub duration: Duration,
}

fn deserialize_duration_internal<'de, D>(data: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let duration = deserialize_duration(data)?;
    if duration.as_secs() == 0 {
        return Err(Error::custom("rate limit duration cannot be 0"));
    }
    Ok(duration)
}

impl From<RateLimitConfig> for parser_sdl::federation::RateLimitConfig {
    fn from(value: RateLimitConfig) -> Self {
        Self {
            limit: value.limit,
            duration: value.duration,
        }
    }
}

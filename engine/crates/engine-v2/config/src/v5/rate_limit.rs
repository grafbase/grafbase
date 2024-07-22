use std::{path::Path, time::Duration};

use super::{PathId, StringId};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SubgraphRateLimitConfig {
    pub limit: usize,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub limit: usize,
    pub duration: Duration,
    pub storage: RateLimitStorage,
    pub redis: RateLimitRedisConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfigRef<'a> {
    pub storage: RateLimitStorage,
    pub redis: RateLimitRedisConfigRef<'a>,
}

impl RateLimitConfig {
    pub fn as_subgraph_config(self) -> SubgraphRateLimitConfig {
        SubgraphRateLimitConfig {
            limit: self.limit,
            duration: self.duration,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum RateLimitStorage {
    #[default]
    InMemory,
    Redis,
}
impl RateLimitStorage {
    pub fn is_redis(&self) -> bool {
        matches!(self, Self::Redis)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRedisConfig {
    pub url: StringId,
    pub key_prefix: StringId,
    pub tls: Option<RateLimitRedisTlsConfig>,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitRedisConfigRef<'a> {
    pub url: &'a str,
    pub key_prefix: &'a str,
    pub tls: Option<RateLimitRedisTlsConfigRef<'a>>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRedisTlsConfig {
    pub cert: PathId,
    pub key: PathId,
    pub ca: Option<PathId>,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitRedisTlsConfigRef<'a> {
    pub cert: &'a Path,
    pub key: &'a Path,
    pub ca: Option<&'a Path>,
}

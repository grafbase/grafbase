#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TrafficShapingConfig {
    pub inflight_deduplication: bool,
}

impl Default for TrafficShapingConfig {
    fn default() -> Self {
        Self {
            inflight_deduplication: true,
        }
    }
}

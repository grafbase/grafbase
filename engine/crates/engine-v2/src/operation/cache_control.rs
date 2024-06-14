use std::time::Duration;

use engine::RequestCacheKey;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OperationCacheControl {
    pub key: RequestCacheKey,
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
}

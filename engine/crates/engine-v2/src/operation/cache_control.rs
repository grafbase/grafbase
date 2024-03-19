use std::time::Duration;

use engine::RequestCacheKey;

#[derive(Debug, Clone)]
pub struct OperationCacheControl {
    pub key: RequestCacheKey,
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
}

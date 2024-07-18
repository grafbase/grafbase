use futures::future::{ready, BoxFuture};
use futures::FutureExt;

use runtime::rate_limiting::{Error, RateLimiter, RateLimiterContext, RateLimiterInner};

pub struct NoopRateLimiter;
impl NoopRateLimiter {
    pub fn runtime() -> RateLimiter {
        RateLimiter::new(NoopRateLimiter)
    }
}
impl RateLimiterInner for NoopRateLimiter {
    fn limit<'a>(&'a self, _context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        ready(Ok(())).boxed()
    }
}

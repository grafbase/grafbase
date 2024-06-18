use futures::future::ready;
use futures::FutureExt;
use runtime::rate_limiting::{Error, RateLimiter, RateLimiterContext};

pub struct NoopRateLimiter;
impl RateLimiter for NoopRateLimiter {
    fn limit<'a>(
        &'a self,
        _context: Box<dyn RateLimiterContext + 'a>,
    ) -> futures::future::BoxFuture<'a, Result<(), Error>> {
        ready(Ok(())).boxed()
    }
}

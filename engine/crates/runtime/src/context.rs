use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};

use futures_util::future::BoxFuture;

pub struct Context {
    request: Arc<dyn RequestContext>,
    pub log: LogContext,
}

pub struct LogContext {
    pub fetch_log_endpoint_url: Option<String>,
    pub request_log_event_id: Option<ulid::Ulid>,
}

impl Context {
    pub fn new(request: &Arc<impl RequestContext + 'static>, log: LogContext) -> Self {
        Self {
            request: Arc::clone(request) as Arc<dyn RequestContext>,
            log,
        }
    }
}

impl Deref for Context {
    type Target = dyn RequestContext;

    fn deref(&self) -> &Self::Target {
        self.request.as_ref()
    }
}

#[async_trait::async_trait]
pub trait RequestContext: Send + Sync {
    fn ray_id(&self) -> &str;
    // Request execution will wait for those futures to end.
    // worker requires a 'static future, so there isn't any choice.
    async fn wait_until(&self, fut: BoxFuture<'static, ()>);
    fn headers(&self) -> &http::HeaderMap;

    fn headers_as_map(&self) -> HashMap<String, String> {
        self.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect()
    }

    fn jwt_claims(&self) -> &BTreeMap<String, serde_json::Value>;
}

pub trait RequestContextExt: RequestContext {
    fn header<H: headers::Header>(&self) -> Option<H> {
        use headers::HeaderMapExt;
        self.headers().typed_get()
    }
}

impl<T: RequestContext> RequestContextExt for T {}

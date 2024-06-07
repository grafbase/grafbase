use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use http::HeaderMap;
use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine::futures_util::future::BoxFuture;
use grafbase_tracing::span::cache::CACHE_SPAN_NAME;
use integration_tests::runtime;
use runtime::cache::{
    CacheInner, CacheMetadata, CacheReadStatus, Cacheable, Entry, EntryState, GlobalCacheConfig, Key,
};
use runtime::context::RequestContext;

struct TestRequestContext;
#[async_trait]
impl RequestContext for TestRequestContext {
    fn ray_id(&self) -> &str {
        unimplemented!()
    }

    async fn wait_until(&self, _fut: BoxFuture<'static, ()>) {
        unimplemented!()
    }

    fn headers(&self) -> &HeaderMap {
        Box::leak(Box::new(HeaderMap::new()))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct TestResponse;
impl Cacheable for TestResponse {
    fn metadata(&self) -> CacheMetadata {
        CacheMetadata::default()
    }
}

#[test]
pub fn cache_status_hit() {
    struct TestCache;
    #[async_trait]
    impl CacheInner for TestCache {
        async fn get(&self, _key: &Key) -> runtime::cache::Result<Entry<Vec<u8>>> {
            Ok(Entry::Hit(
                serde_json::to_vec(&TestResponse).unwrap(),
                Duration::from_millis(500),
            ))
        }

        async fn put(
            &self,
            _key: &Key,
            _state: EntryState,
            _value: Vec<u8>,
            _metadata: CacheMetadata,
        ) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _key: &Key) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn purge_by_tags(&self, _tags: Vec<String>) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn purge_by_hostname(&self, _hostname: String) -> runtime::cache::Result<()> {
            unimplemented!()
        }
    }

    runtime().block_on(async {
        // prepare
        let cache = runtime::cache::Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                ..Default::default()
            },
        );
        let cache_key = Key::unchecked_new(String::new());
        let execution = async { Ok::<_, Infallible>(Arc::new(TestResponse)) };
        let request_context = TestRequestContext;

        let cache_span = expect::span().at_level(Level::INFO).named(CACHE_SPAN_NAME);
        let (subscriber, handle) = subscriber::mock()
            .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
            .new_span(
                cache_span
                    .clone()
                    .with_field(expect::field("cache.status").with_value(&"BYPASS")),
            )
            .enter(cache_span.clone())
            .record(cache_span.clone(), expect::field("cache.status").with_value(&"HIT"))
            .run_with_handle();

        let _default = tracing::subscriber::set_default(subscriber);

        // act
        let cached_response = cache
            .cached_execution(&request_context, cache_key, execution)
            .await
            .unwrap();

        // assert
        assert_eq!(cached_response.read_status(), CacheReadStatus::Hit);
        handle.assert_finished();
    });
}

#[test]
pub fn cache_status_error() {
    struct TestCache;
    #[async_trait]
    impl CacheInner for TestCache {
        async fn get(&self, _key: &Key) -> runtime::cache::Result<Entry<Vec<u8>>> {
            Ok(Entry::Miss)
        }

        async fn put(
            &self,
            _key: &Key,
            _state: EntryState,
            _value: Vec<u8>,
            _metadata: CacheMetadata,
        ) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _key: &Key) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn purge_by_tags(&self, _tags: Vec<String>) -> runtime::cache::Result<()> {
            unimplemented!()
        }

        async fn purge_by_hostname(&self, _hostname: String) -> runtime::cache::Result<()> {
            unimplemented!()
        }
    }

    runtime().block_on(async {
        // prepare
        let cache = runtime::cache::Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                ..Default::default()
            },
        );
        let cache_key = Key::unchecked_new(String::new());
        let execution = async { Err::<Arc<TestResponse>, _>(runtime::cache::Error::Origin(String::new())) };
        let request_context = TestRequestContext;

        let cache_span = expect::span().at_level(Level::INFO).named(CACHE_SPAN_NAME);
        let (subscriber, handle) = subscriber::mock()
            .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
            .new_span(
                cache_span
                    .clone()
                    .with_field(expect::field("cache.status").with_value(&"BYPASS")),
            )
            .enter(cache_span.clone())
            .record(cache_span.clone(), expect::field("cache.is_error").with_value(&true))
            .run_with_handle();

        let _default = tracing::subscriber::set_default(subscriber);

        // act
        let cached_response = cache.cached_execution(&request_context, cache_key, execution).await;

        // assert
        assert!(cached_response.is_err());
        handle.assert_finished();
    });
}

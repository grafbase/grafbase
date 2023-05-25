#[cfg(not(feature = "local"))]
pub mod remote {
    use crate::cache::{CacheError, CacheResult, GlobalCacheProvider};
    use crate::platform::config;
    use async_trait::async_trait;
    use js_sys::Uint8Array;
    use worker::{Fetch, Headers, Method, RequestInit};

    const CLOUDFLARE_BASE_API_URL: &str = "https://api.cloudflare.com/client/v4";

    pub struct CloudflareGlobal {
        cloudflare_config: config::CloudflareConfig,
    }

    impl CloudflareGlobal {
        pub fn new(cloudflare_config: config::CloudflareConfig) -> Self {
            CloudflareGlobal { cloudflare_config }
        }
    }

    #[async_trait(?Send)]
    impl GlobalCacheProvider for CloudflareGlobal {
        // when we start getting rate limited, this implementation needs to be revisited
        // change to something that keeps us within the 700 rps / 2500 burst
        async fn purge_by_tags(&self, tags: Vec<String>) -> CacheResult<()> {
            let uri = [
                CLOUDFLARE_BASE_API_URL,
                "zones",
                &self.cloudflare_config.zone_id,
                "purge_cache",
            ]
            .join("/");

            let mut headers = Headers::new();
            headers
                .set(
                    http::header::AUTHORIZATION.as_str(),
                    &format!("Bearer {}", &self.cloudflare_config.api_key),
                )
                .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?;

            #[derive(serde::Serialize)]
            struct CloudflareCachePurgeRequest {
                pub tags: Vec<String>,
            }

            let bytes = serde_json::to_vec(&CloudflareCachePurgeRequest { tags })
                .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?;

            let mut init = RequestInit::new();
            init.with_body(Some(Uint8Array::from(bytes.as_slice()).into()))
                .with_method(Method::Post)
                .with_headers(headers);

            match Fetch::Request(
                worker::Request::new_with_init(&uri, &init)
                    .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?,
            )
            .send()
            .await
            {
                Ok(mut response) => {
                    let body = response
                        .text()
                        .await
                        .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?;

                    if response.status_code() != http::StatusCode::OK.as_u16() {
                        return Err(CacheError::CachePurgeByTags(body));
                    }
                    Ok(())
                }
                Err(err) => Err(CacheError::CachePurgeByTags(err.to_string())),
            }
        }
    }
}

#[cfg(feature = "local")]
pub mod noop {
    use crate::cache::{CacheResult, GlobalCacheProvider};
    use async_trait::async_trait;

    pub struct NoopGlobalCache;

    #[async_trait(?Send)]
    impl GlobalCacheProvider for NoopGlobalCache {
        async fn purge_by_tags(&self, _tags: Vec<String>) -> CacheResult<()> {
            unimplemented!()
        }
    }
}

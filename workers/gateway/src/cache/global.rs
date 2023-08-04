#[cfg(all(not(feature = "local"), not(feature = "sqlite")))]
pub mod remote {
    use async_trait::async_trait;
    use js_sys::Uint8Array;
    use serde::Serialize;
    use worker::{Fetch, Headers, Method, RequestInit, Response};

    use crate::{
        cache::{CacheError, CacheResult, GlobalCacheProvider},
        platform::config,
    };

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
            #[derive(serde::Serialize)]
            struct CloudflareCachePurgeRequest {
                pub tags: Vec<String>,
            }

            let (uri, request_init) = self.get_request(CloudflareCachePurgeRequest { tags })?;

            let response = Fetch::Request(
                worker::Request::new_with_init(&uri, &request_init)
                    .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?,
            )
            .send()
            .await;

            self.handle_response(response).await
        }

        // when we start getting rate limited, this implementation needs to be revisited
        // change to something that keeps us within the 700 rps / 2500 burst
        async fn purge_by_hostname(&self, hostname: String) -> CacheResult<()> {
            #[derive(serde::Serialize)]
            struct CloudflareCachePurgeRequest {
                pub hosts: Vec<String>,
            }

            let (uri, request_init) = self.get_request(CloudflareCachePurgeRequest { hosts: vec![hostname] })?;

            let response = Fetch::Request(
                worker::Request::new_with_init(&uri, &request_init)
                    .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?,
            )
            .send()
            .await;

            self.handle_response(response).await
        }
    }

    impl CloudflareGlobal {
        fn get_request<T: Serialize>(&self, body: T) -> CacheResult<(String, RequestInit)> {
            let uri = [
                CLOUDFLARE_BASE_API_URL,
                "zones",
                &self.cloudflare_config.zone_id,
                "purge_cache",
            ]
            .join("/");

            let mut headers = Headers::new();
            use secrecy::ExposeSecret;
            headers
                .set(
                    http::header::AUTHORIZATION.as_str(),
                    &format!("Bearer {}", self.cloudflare_config.api_key.expose_secret()),
                )
                .map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?;

            let bytes = serde_json::to_vec(&body).map_err(|err| CacheError::CachePurgeByTags(err.to_string()))?;

            let mut init = RequestInit::new();
            init.with_body(Some(Uint8Array::from(bytes.as_slice()).into()))
                .with_method(Method::Post)
                .with_headers(headers);

            Ok((uri, init))
        }

        async fn handle_response(&self, response: worker::Result<Response>) -> CacheResult<()> {
            match response {
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

#[cfg(any(feature = "local", feature = "sqlite", test))]
pub mod noop {
    use async_trait::async_trait;

    use crate::cache::{CacheResult, GlobalCacheProvider};

    pub struct NoopGlobalCache;

    #[async_trait(?Send)]
    impl GlobalCacheProvider for NoopGlobalCache {
        async fn purge_by_tags(&self, _tags: Vec<String>) -> CacheResult<()> {
            unimplemented!()
        }

        async fn purge_by_hostname(&self, _hostname: String) -> CacheResult<()> {
            unimplemented!()
        }
    }
}
